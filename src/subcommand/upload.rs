use {super::*, reqwest::blocking::Body};

struct Context {
  archive: Archive,
  client: Client,
  files: u64,
  files_uploaded: u64,
  missing: HashSet<Hash>,
  path: Utf8PathBuf,
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
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL", value_parser = CheckedUrl::check)]
  server: Url,
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

  fn upload_directory(context: &mut Context, file_path: &Utf8Path, hash: Hash) -> Result {
    let error_context = error::UnarchiveManifest {
      path: &context.path,
    };

    let cbor = context.archive.file(hash).context(error_context)?;

    let directory = Directory::decode_from_slice(cbor)
      .context(archive_error::DirectoryDecode)
      .context(error_context)?;

    context.client.put_file(hash, cbor.to_vec().into())?;

    for (component, entry) in &directory.entries {
      let file_path = file_path.join(component);
      match entry {
        Entry::Directory { hash, .. } => Self::upload_directory(context, &file_path, *hash)?,
        Entry::File { hash, .. } => {
          if context.missing.contains(hash) {
            Self::upload_package_file(context, entry, &file_path)?;
            context.files_uploaded += 1;
            context
              .progress_bar
              .set_message(progress_bar::file_progress_message(
                context.files_uploaded,
                context.files,
              ));
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

    let bar = progress_bar::new(options, size);

    let file = filesystem::open(input)?;

    let body = Body::sized(bar.wrap_read(file), size);

    client.put_file(hash, body)?;

    bar.finish();

    Ok(())
  }

  fn upload_package(&self, options: Options, client: Client) -> Result {
    let (path, archive) = Archive::load_with_opt_path(self.input.as_deref())?;

    let error_context = error::UnarchiveManifest { path: &path };

    let fingerprint = archive.fingerprint().context(error_context)?;

    if client.has_package(fingerprint)? {
      if !options.quiet {
        eprintln!("server already has package");
      }

      return Ok(());
    }

    let manifest = archive.unpack().context(error_context)?;

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

    let progress_bar = progress_bar::with_message(
      &options,
      bytes,
      progress_bar::file_progress_message(0, files),
    );

    let mut context = Context {
      archive,
      progress_bar,
      client,
      files_uploaded: 0,
      missing,
      path,
      files,
    };

    let root = context.path.parent().unwrap().to_owned();

    Self::upload_directory(&mut context, &root, fingerprint.into())?;

    context.client.verify_package(fingerprint)?;

    context.progress_bar.finish();

    Ok(())
  }

  fn upload_package_file(context: &Context, expected: &Entry, path: &Utf8Path) -> Result {
    let file = filesystem::open(path)?;

    let body = Body::sized(context.progress_bar.wrap_read(file), expected.size());

    context.client.put_file(expected.hash(), body)?;

    Ok(())
  }
}
