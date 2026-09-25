use {super::*, reqwest::blocking::Body};

struct Context {
  client: Client,
  loader: Loader,
  missing: HashSet<Hash>,
  progress_bar: ProgressBar,
}

#[derive(Parser)]
pub(crate) struct Upload {
  #[arg(help = "Authenticate with key <KEY>", long, value_name = "KEY")]
  auth: Option<KeyName>,
  #[arg(help = "Upload file instead of package", long)]
  file: bool,
  #[arg(
    help = "Upload <PATH>, defaults to current directory for packages",
    required_if_eq("file", "true"),
    value_name = "PATH"
  )]
  input: Option<Utf8PathBuf>,
  #[arg(
    conflicts_with = "file",
    help = "Replace package number <NUMBER>",
    long,
    value_name = "NUMBER"
  )]
  replace: Option<u64>,
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL")]
  server: ServerUrl,
  #[arg(
    conflicts_with_all = ["file", "replace"],
    help = "Update package number <NUMBER>",
    long,
    value_name = "NUMBER"
  )]
  update: Option<u64>,
}

impl Upload {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = Client::new(&options, self.server.clone(), self.auth.as_ref())?;

    if self.file {
      self.upload_file(&options, &client)
    } else {
      self.upload_package(options, client)
    }
  }

  fn upload_directory(
    context: &mut Context,
    file_path: &Utf8Path,
    hash: Hash,
    size: u64,
  ) -> Result {
    let error_context = error::UnarchiveManifest {
      path: context.loader.path(),
    };

    let deco = context
      .loader
      .archive()
      .file(hash, size)
      .context(error_context)?;

    let directory = Directory::decode_from_slice(deco)
      .context(archive_error::DirectoryDecode)
      .context(error_context)?;

    context.client.put_file(hash, deco.to_vec().into())?;

    for (component, entry) in &directory.entries {
      let file_path = file_path.join(component);
      match entry {
        Entry::Directory { hash, size, .. } => {
          Self::upload_directory(context, &file_path, *hash, *size)?;
        }
        Entry::File { hash, .. } => {
          if context.missing.contains(hash) {
            Self::upload_package_file(context, entry, &file_path)?;
            context.progress_bar.item_done();
          }
        }
      }
    }

    context.client.verify_directory(hash)?;

    Ok(())
  }

  fn upload_file(&self, options: &Options, client: &Client) -> Result {
    let input = self.input.as_deref().unwrap();

    let File { hash, size } = options
      .hash_file(input)
      .context(error::FilesystemIo { path: input })?;

    let bar = ProgressBar::bytes(options, size);

    let file = filesystem::open(input)?;

    let body = Body::sized(bar.wrap_read(file), size);

    client.put_file(hash, body)?;

    Ok(())
  }

  fn upload_package(&self, options: Options, client: Client) -> Result {
    let loader = Loader::load(self.input.as_deref())?;

    let package = loader.package()?;

    let fingerprint = Fingerprint(package.hash());

    let previous = if let Some(number) = self.update {
      let head = client.number(number)?;

      if head.package == fingerprint {
        if !options.quiet {
          eprintln!("package number {number} is up to date");
        }
        return Ok(());
      }

      Some(head.revision)
    } else {
      None
    };

    let revision_object = RevisionObject {
      version: Version::Zero,
      package: fingerprint,
      previous,
    };

    let revision = revision_object.hash();

    if self.replace.is_none() && self.update.is_none() && client.is_head(revision)? {
      if !options.quiet {
        eprintln!("server already has package");
      }
      return Ok(());
    }

    let manifest = loader.unpack()?;

    let manifest_files = manifest.files();

    let hashes = manifest_files
      .values()
      .map(|file| file.hash)
      .collect::<BTreeSet<Hash>>();

    let missing = client.missing_files(hashes)?;

    let mut files = 0;

    let mut bytes = 0;

    for file in manifest_files.values() {
      if missing.contains(&file.hash) {
        files += 1;
        bytes += file.size;
      }
    }

    if !options.quiet {
      eprintln!(
        "uploading {files} of {}",
        Count::new(manifest_files.len(), "file")
      );
    }

    let mut context = Context {
      client,
      loader,
      missing,
      progress_bar: ProgressBar::items(&options, bytes, files, "files"),
    };

    let root = context.loader.path().parent().unwrap().to_owned();

    Self::upload_directory(&mut context, &root, package.hash(), package.size())?;

    context.client.verify_package(fingerprint)?;

    context
      .client
      .put_file(revision.into(), revision_object.encode_to_vec().into())?;

    let (mode, verb) = match (self.replace, self.update) {
      (Some(number), None) => (api::revision::Mode::Replace { number }, "replaced"),
      (None, Some(number)) => (api::revision::Mode::Update { number }, "updated"),
      (None, None) => (api::revision::Mode::New, "created"),
      (Some(_), Some(_)) => unreachable!(),
    };

    let number = context
      .client
      .verify_revision(revision, api::revision::Request { mode })?;

    if !options.quiet {
      eprintln!("{verb} package number {number}");
    }

    Ok(())
  }

  fn upload_package_file(context: &Context, expected: &Entry, path: &Utf8Path) -> Result {
    let file = filesystem::open(path)?;

    let body = Body::sized(context.progress_bar.wrap_read(file), expected.size());

    context.client.put_file(expected.hash(), body)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn file_requires_path() {
    assert_missing_argument::<Upload>(&["--server", "http://127.0.0.1:1", "--file"], &["<PATH>"]);
  }

  #[test]
  fn replace_conflicts_with_file() {
    assert_argument_conflict::<Upload>(
      &[
        "--server",
        "http://127.0.0.1:1",
        "--file",
        "--replace",
        "1",
        "foo",
      ],
      "--file",
      "--replace <NUMBER>",
    );
  }

  #[test]
  fn server_url_must_be_http_or_https() {
    assert_invalid_argument_value::<Upload>(
      &["--server", "ftp://example.com"],
      "--server <URL>",
      "ftp://example.com",
      "URL scheme `ftp` not allowed, must be `http` or `https`",
    );
  }

  #[test]
  fn update_conflicts_with_replace() {
    assert_argument_conflict::<Upload>(
      &[
        "--server",
        "http://127.0.0.1:1",
        "--replace",
        "1",
        "--update",
        "1",
      ],
      "--replace <NUMBER>",
      "--update <NUMBER>",
    );
  }
}
