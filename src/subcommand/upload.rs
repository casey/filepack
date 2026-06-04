use {
  super::*,
  reqwest::blocking::{Body, RequestBuilder},
  url::Host,
};

struct Context {
  archive: Archive,
  client: Client,
  files: u64,
  files_uploaded: u64,
  key: Option<PrivateKey>,
  missing: HashSet<Hash>,
  options: Options,
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
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL", value_parser = parse_server_url)]
  server: Url,
}

impl Upload {
  fn request_with_token(
    &self,
    mut builder: RequestBuilder,
    key: Option<&PrivateKey>,
  ) -> Result<RequestBuilder> {
    if let Some(key) = key {
      let host = self.server.host_str().unwrap().to_owned();
      builder = builder.bearer_auth(Token::encode(key, &host)?);
    }

    Ok(builder)
  }

  pub(crate) fn run(self, options: Options) -> Result {
    let key = if let Some(name) = &self.auth {
      let loopback = match self.server.host().unwrap() {
        Host::Domain(domain) => domain == "localhost",
        Host::Ipv4(addr) => addr.is_loopback(),
        Host::Ipv6(addr) => addr.is_loopback(),
      };

      ensure!(
        self.server.scheme() == "https" || loopback,
        error::TokenOverHttp,
      );

      let keychain = Keychain::load(&options)?;

      Some(PrivateKey::load(
        &keychain.path.join(name.private_key_filename()),
      )?)
    } else {
      None
    };

    if self.file {
      self.upload_file(&options, key.as_ref())
    } else {
      self.upload_package(options, key)
    }
  }

  fn upload_body(
    &self,
    client: &Client,
    hash: Hash,
    body: Body,
    key: Option<&PrivateKey>,
  ) -> Result {
    let url = self.server.join(&format!("file/{hash}")).unwrap();
    let request = client.put(url).body(body);
    self
      .request_with_token(request, key)?
      .send()
      .check_status()?;
    Ok(())
  }

  fn upload_directory(&self, context: &mut Context, file_path: &Utf8Path, hash: Hash) -> Result {
    let error_context = error::UnarchiveManifest {
      path: &context.path,
    };

    let cbor = context.archive.file(hash).context(error_context)?;

    let directory = Directory::decode_from_slice(cbor)
      .context(archive_error::DirectoryDecode)
      .context(error_context)?;

    self.upload_body(
      &context.client,
      hash,
      cbor.to_vec().into(),
      context.key.as_ref(),
    )?;

    for (component, entry) in &directory.entries {
      let file_path = file_path.join(component);
      match entry.ty {
        EntryType::Directory => {
          self.upload_directory(context, &file_path, entry.hash)?;
        }
        EntryType::File => {
          if context.missing.contains(&entry.hash) {
            self.upload_package_file(context, entry, &file_path)?;
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

    let url = self.server.join(&format!("directory/{hash}")).unwrap();
    let request = context.client.post(url);
    self
      .request_with_token(request, context.key.as_ref())?
      .send()
      .check_status()?;

    Ok(())
  }

  fn upload_file(&self, options: &Options, key: Option<&PrivateKey>) -> Result {
    let input = self.input.as_deref().unwrap();

    let File { hash, size } = options
      .hash_file(input)
      .context(error::FilesystemIo { path: input })?;

    let bar = progress_bar::new(options, size);

    let file = filesystem::open(input)?;

    let body = Body::sized(bar.wrap_read(file), size);

    let client = client()?;

    self.upload_body(&client, hash, body, key)?;

    bar.finish();

    Ok(())
  }

  fn upload_package(&self, options: Options, key: Option<PrivateKey>) -> Result {
    let (path, archive) = Archive::load_with_opt_path(self.input.as_deref())?;

    let error_context = error::UnarchiveManifest { path: &path };

    let fingerprint = archive.fingerprint().context(error_context)?;

    let client = client()?;

    let url = self.server.join(&format!("package/{fingerprint}")).unwrap();

    if client.head(url).send().found()?.is_some() {
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

    let body = api::missing::Request {
      hashes: hashes.into(),
    }
    .encode_to_vec();

    let url = self.server.join("missing").unwrap();

    let missing = self
      .request_with_token(client.post(url).body(body), key.as_ref())?
      .send()
      .check_status()?
      .cbor::<api::missing::Response>()?
      .hashes
      .iter()
      .copied()
      .collect::<HashSet<Hash>>();

    let mut files = 0;

    let mut bytes = 0;

    for file in manifest_files.values() {
      if missing.contains(&file.hash) {
        files += 1;
        bytes += file.size;
      }
    }

    if !options.quiet {
      eprintln!("uploading {files} of {} files", manifest_files.len());
    }

    let progress_bar = progress_bar::with_files(&options, bytes, files);

    let mut context = Context {
      archive,
      progress_bar,
      client,
      files_uploaded: 0,
      key,
      missing,
      options,
      path,
      files,
    };

    let root = context.path.parent().unwrap().to_owned();

    self.upload_directory(&mut context, &root, fingerprint.into())?;

    let url = self.server.join(&format!("package/{fingerprint}")).unwrap();
    let request = context.client.post(url);
    self
      .request_with_token(request, context.key.as_ref())?
      .send()
      .check_status()?;

    context.progress_bar.finish();

    Ok(())
  }

  fn upload_package_file(&self, context: &Context, expected: &Entry, path: &Utf8Path) -> Result {
    let actual = context
      .options
      .hash_file(path)
      .context(error::FilesystemIo { path })?;

    let expected = File {
      hash: expected.hash,
      size: expected.size,
    };

    if actual != expected {
      File::eprint_mismatch(actual, expected, path.as_ref());
      return Err(error::FileMismatch { path }.build());
    }

    let file = filesystem::open(path)?;

    let body = Body::sized(context.progress_bar.wrap_read(file), expected.size);

    self.upload_body(&context.client, expected.hash, body, context.key.as_ref())?;

    Ok(())
  }
}
