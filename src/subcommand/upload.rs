use {
  super::*,
  reqwest::blocking::{Body, RequestBuilder},
  std::cell::Cell,
  url::Host,
};

struct Context<'a> {
  archive: &'a Archive,
  archive_path: &'a Utf8Path,
  bar: &'a ProgressBar,
  client: &'a Client,
  files_uploaded: Cell<u64>,
  key: Option<&'a PrivateKey>,
  options: &'a Options,
  total_files: u64,
}

#[derive(Parser)]
pub(crate) struct Upload {
  #[arg(help = "Authenticate with key <KEY>", long, value_name = "KEY")]
  auth: Option<KeyName>,
  #[arg(help = "Upload file instead of package", long)]
  file: bool,
  #[arg(help = "Upload <PATH>", value_name = "PATH")]
  input: Utf8PathBuf,
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

    let client = Client::new();

    if self.file {
      self.upload_file(&client, &self.input, &options, key.as_ref())
    } else {
      self.upload_package(&client, &self.input, &options, key.as_ref())
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

  fn upload_directory(&self, ctx: &Context, file_path: &Utf8Path, hash: Hash) -> Result {
    let error_context = error::UnarchiveManifest {
      path: ctx.archive_path,
    };

    let cbor = ctx.archive.file(hash).context(error_context)?;

    let directory = Directory::decode_from_slice(cbor)
      .context(archive_error::DirectoryDecode)
      .context(error_context)?;

    self.upload_body(ctx.client, hash, cbor.to_vec().into(), ctx.key)?;

    for (component, entry) in &directory.entries {
      let file_path = file_path.join(component);
      match entry.ty {
        EntryType::Directory => {
          self.upload_directory(ctx, &file_path, entry.hash)?;
        }
        EntryType::File => {
          self.upload_package_file(ctx, entry, &file_path)?;
          ctx.files_uploaded.set(ctx.files_uploaded.get() + 1);
          ctx.bar.set_message(format!(
            "{}/{} files",
            ctx.files_uploaded.get(),
            ctx.total_files,
          ));
        }
      }
    }

    let url = self.server.join(&format!("directory/{hash}")).unwrap();
    let request = ctx.client.post(url);
    self
      .request_with_token(request, ctx.key)?
      .send()
      .check_status()?;

    Ok(())
  }

  fn upload_file(
    &self,
    client: &Client,
    path: &Utf8Path,
    options: &Options,
    key: Option<&PrivateKey>,
  ) -> Result {
    let File { hash, size } = options
      .hash_file(path)
      .context(error::FilesystemIo { path })?;

    let bar = progress_bar::with_files(options, size, 1);

    let file = filesystem::open(path)?;

    let body = Body::sized(bar.wrap_read(file), size);

    self.upload_body(client, hash, body, key)?;

    bar.set_message("1/1 files".to_owned());
    bar.finish();

    Ok(())
  }

  fn upload_package(
    &self,
    client: &Client,
    archive_path: &Utf8Path,
    options: &Options,
    key: Option<&PrivateKey>,
  ) -> Result {
    let archive = Archive::load_with_path(archive_path, archive_path)?;

    let manifest = archive
      .unpack()
      .context(error::UnarchiveManifest { path: archive_path })?;

    let fingerprint = manifest.fingerprint();

    let total_files = manifest.files().len().into_u64();
    let total_bytes = u64::try_from(manifest.total_size()).unwrap_or(u64::MAX);

    let bar = progress_bar::with_files(options, total_bytes, total_files);

    let ctx = Context {
      archive: &archive,
      archive_path,
      bar: &bar,
      client,
      files_uploaded: Cell::new(0),
      key,
      options,
      total_files,
    };

    self.upload_directory(&ctx, archive_path.parent().unwrap(), fingerprint.into())?;

    let url = self.server.join(&format!("package/{fingerprint}")).unwrap();
    let request = client.post(url);
    self
      .request_with_token(request, key)?
      .send()
      .check_status()?;

    bar.finish();

    Ok(())
  }

  fn upload_package_file(&self, ctx: &Context, expected: &Entry, path: &Utf8Path) -> Result {
    let actual = ctx
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

    let body = Body::sized(ctx.bar.wrap_read(file), expected.size);

    self.upload_body(ctx.client, expected.hash, body, ctx.key)?;

    Ok(())
  }
}
