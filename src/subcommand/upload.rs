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
  #[arg(group = "mode", help = "Upload file instead of package", long)]
  file: bool,
  #[arg(
    help = "Upload <PATH>, defaults to current directory for packages",
    required_if_eq("file", "true"),
    value_hint = ValueHint::AnyPath,
    value_name = "PATH"
  )]
  input: Option<Utf8PathBuf>,
  #[arg(group = "mode", help = "Upload package without revision", long)]
  no_revision: bool,
  #[arg(
    conflicts_with_all = ["file", "no_revision"],
    help = "Upload to package number <NUMBER>",
    long,
    value_name = "NUMBER"
  )]
  number: Option<u64>,
  #[arg(
    conflicts_with_all = ["file", "no_revision"],
    help = "Replace package regardless of revision history",
    long
  )]
  replace: bool,
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL")]
  server: ServerUrl,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

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

    if context.missing.contains(&hash) {
      tracing::debug!("uploading directory `{file_path}`");
      context.client.put_file(hash, deco.to_vec().into())?;
    }

    for (component, entry) in &directory.entries {
      let file_path = file_path.join(component);
      match entry.ty() {
        EntryType::Directory => {
          Self::upload_directory(context, &file_path, entry.hash, entry.size)?;
        }
        EntryType::File => {
          if context.missing.contains(&entry.hash) {
            tracing::debug!("uploading file `{file_path}`");
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
      .context(filesystem_error::Io { path: input })?;

    let bar = ProgressBar::bytes(options, size);

    let file = filesystem::open(input)?;

    let body = Body::sized(bar.wrap_read(file), size);

    client.put_file(hash, body)?;

    Ok(())
  }

  fn upload_package(&self, options: Options, client: Client) -> Result {
    let loader = Loader::load(self.input.as_deref())?;

    let root = loader.root().to_owned();

    let mut state = State::load(&root)?;

    let server = self.server.to_string();

    let package = loader.package()?;

    let number = self.number.or_else(|| {
      state
        .servers
        .get(&server)
        .map(|server_state| server_state.number)
    });

    let manifest = loader.unpack()?;

    let revision = loader.revision_hash()?;

    let parents = manifest
      .parents
      .as_ref()
      .map_or_default(NonEmptyOrderedSet::as_slice);

    for signature in manifest.signatures() {
      signature.verify(revision)?;
    }

    let mode = if self.no_revision {
      let context = Self::upload_package_contents(&options, client, loader, &manifest, package)?;

      if !options.quiet {
        eprintln!(
          "uploaded {}",
          context.client.package_url(package.hash.into()),
        );
      }

      return Ok(());
    } else if self.replace {
      let number = number.context(error::ReplaceNumberMissing)?;

      for &parent in parents {
        client
          .revision_opt(parent)?
          .context(error::RevisionParentUnpublished {
            parent,
            server: &server,
          })?;
      }

      api::revision::post::Mode::Replace { number }
    } else if let Some(number) = number {
      let head = client.number(number)?.revision;

      if head == revision {
        state
          .servers
          .insert(server, ServerState { number, revision });

        state.save(&root)?;

        if !options.quiet {
          eprintln!("package number {number} is up to date");
        }

        Self::upload_signatures(&options, &client, &manifest, revision)?;

        return Ok(());
      }

      ensure! {
        parents.first() == Some(&head),
        error::RevisionStale {
          head,
          number,
          revision,
        },
      }

      api::revision::post::Mode::Update { number }
    } else if let Some(response) = client.revision_opt(revision)?
      && let Some(&number) = response.numbers.first()
    {
      ensure! {
        response.numbers.len() == 1,
        error::RevisionNumberAmbiguous {
          numbers: response.numbers.iter().copied().collect::<BTreeSet<u64>>(),
        },
      }

      state
        .servers
        .insert(server, ServerState { number, revision });

      state.save(&root)?;

      if !options.quiet {
        eprintln!("server already has this revision as package number {number}");
      }

      Self::upload_signatures(&options, &client, &manifest, revision)?;

      return Ok(());
    } else if let Some(&parent) = parents.first() {
      let response = client
        .revision_opt(parent)?
        .context(error::RevisionParentUnpublished {
          parent,
          server: &server,
        })?;

      ensure! {
        response.numbers.len() <= 1,
        error::RevisionParentNumberAmbiguous {
          numbers: response.numbers.iter().copied().collect::<BTreeSet<u64>>(),
          parent,
          server,
        },
      }

      let number = *response
        .numbers
        .first()
        .context(error::RevisionParentNotHead {
          parent,
          server: &server,
        })?;

      api::revision::post::Mode::Update { number }
    } else {
      api::revision::post::Mode::New
    };

    let context = Self::upload_package_contents(&options, client, loader, &manifest, package)?;

    context
      .client
      .put_file(revision.into(), context.loader.revision()?.to_vec().into())?;

    let number = context
      .client
      .verify_revision(revision, api::revision::post::Request { mode })?;

    state
      .servers
      .insert(server, ServerState { number, revision });

    state.save(&root)?;

    if !options.quiet {
      eprintln!("{} package number {number}", mode.verb());
    }

    Self::upload_signatures(&options, &context.client, &manifest, revision)?;

    Ok(())
  }

  fn upload_package_contents(
    options: &Options,
    client: Client,
    loader: Loader,
    manifest: &Manifest,
    package: Entry,
  ) -> Result<Context> {
    let manifest_files = manifest.files();

    let missing = client.missing_files(loader.hashes()?)?;

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

    let root = loader.root().to_owned();

    let mut context = Context {
      client,
      loader,
      missing,
      progress_bar: ProgressBar::items(options, bytes, files, "files"),
    };

    Self::upload_directory(&mut context, &root, package.hash, package.size)?;

    context.client.verify_package(package.hash.into())?;

    Ok(context)
  }

  fn upload_package_file(context: &Context, expected: &Entry, path: &Utf8Path) -> Result {
    let file = filesystem::open(path)?;

    let body = Body::sized(context.progress_bar.wrap_read(file), expected.size);

    context.client.put_file(expected.hash, body)?;

    Ok(())
  }

  fn upload_signatures(
    options: &Options,
    client: &Client,
    manifest: &Manifest,
    revision: RevisionHash,
  ) -> Result {
    let present = client.revision(revision)?.signatures;

    let mut uploaded = 0;

    for signature in manifest.signatures() {
      let signature = signature.encode_to_vec();

      let hash = Hash::bytes(&signature);

      if present.contains(&hash) {
        continue;
      }

      client.put_file(hash, signature.into())?;

      client.verify_signature(hash)?;

      uploaded += 1;
    }

    if !options.quiet {
      let unknown = manifest.unknown_signatures();

      if unknown > 0 {
        eprintln!("ignored {}", Count::new(unknown, "unrecognized signature"));
      }

      if uploaded > 0 {
        eprintln!("uploaded {}", Count::new(uploaded, "signature"));
      }
    }

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
  fn number_conflicts_with_file() {
    assert_argument_conflict::<Upload>(
      &[
        "--server",
        "http://127.0.0.1:1",
        "--file",
        "--number",
        "1",
        "foo",
      ],
      "--file",
      "--number <NUMBER>",
    );
  }

  #[test]
  fn number_conflicts_with_no_revision() {
    assert_argument_conflict::<Upload>(
      &[
        "--server",
        "http://127.0.0.1:1",
        "--no-revision",
        "--number",
        "1",
      ],
      "--no-revision",
      "--number <NUMBER>",
    );
  }

  #[test]
  fn replace_conflicts_with_file() {
    assert_argument_conflict::<Upload>(
      &[
        "--server",
        "http://127.0.0.1:1",
        "--file",
        "--replace",
        "foo",
      ],
      "--file",
      "--replace",
    );
  }

  #[test]
  fn replace_conflicts_with_no_revision() {
    assert_argument_conflict::<Upload>(
      &[
        "--server",
        "http://127.0.0.1:1",
        "--no-revision",
        "--replace",
      ],
      "--no-revision",
      "--replace",
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
}
