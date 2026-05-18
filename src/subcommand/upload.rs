use {
  super::*,
  reqwest::blocking::{Body, Client},
};

#[derive(Parser)]
pub(crate) struct Upload {
  #[arg(
    help = "Trust additional CA certificate at <PATH> (PEM-encoded)",
    long,
    value_name = "PATH"
  )]
  ca_cert: Option<Utf8PathBuf>,
  #[arg(help = "Upload file instead of package", long)]
  file: bool,
  #[arg(help = "Upload <PATH>", value_name = "PATH")]
  input: Utf8PathBuf,
  #[arg(
    help = "Authenticate uploads with <KEY> from the keychain",
    long,
    value_name = "KEY"
  )]
  key: Option<KeyName>,
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL", value_parser = parse_server_url)]
  server: Url,
}

impl Upload {
  fn build_client(&self, options: &Options) -> Result<Client> {
    let mut builder = Client::builder().use_rustls_tls();

    if let Some(name) = &self.key {
      let keychain = Keychain::load(options)?;
      let private = keychain.private_key(name)?;
      let (cert_pem, key_pem) = tls::self_signed_cert(&private);
      let identity =
        reqwest::Identity::from_pem(format!("{cert_pem}{key_pem}").as_bytes()).unwrap();
      builder = builder.identity(identity);
    }

    if let Some(path) = &self.ca_cert {
      let pem = fs::read(path).context(error::FilesystemIo { path })?;
      let cert = reqwest::Certificate::from_pem(&pem).unwrap();
      builder = builder.add_root_certificate(cert);
    }

    builder.build().context(error::Request)
  }

  pub(crate) fn run(self, options: Options) -> Result {
    let client = self.build_client(&options)?;

    if self.file {
      self.upload_file(&client, &self.input, &options)
    } else {
      self.upload_package(&client, &self.input, &options)
    }
  }

  fn upload_body(&self, client: &Client, hash: Hash, body: Body) -> Result {
    let url = self.server.join(&hash.to_string()).unwrap();
    client.put(url).body(body).send().check_status()?;
    Ok(())
  }

  fn upload_file(&self, client: &Client, path: &Utf8Path, options: &Options) -> Result {
    let hash = options
      .hash_file(path)
      .context(error::FilesystemIo { path })?
      .hash;

    let file = filesystem::open(path)?;

    self.upload_body(client, hash, file.into())?;

    Ok(())
  }

  fn upload_package(&self, client: &Client, archive_path: &Utf8Path, options: &Options) -> Result {
    let archive = Archive::load_with_path(archive_path, archive_path)?;

    let context = error::UnarchiveManifest { path: archive_path };

    let fingerprint = archive.fingerprint().context(context)?;

    let mut directories = vec![(
      fingerprint.into(),
      archive_path.parent().unwrap().to_owned(),
    )];

    while let Some((hash, path)) = directories.pop() {
      let cbor = archive.file(hash).context(context)?;

      let directory = Directory::decode_from_slice(cbor)
        .context(archive_error::DirectoryDecode)
        .context(context)?;

      self.upload_body(client, hash, cbor.to_vec().into())?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry.ty {
          EntryType::Directory => directories.push((entry.hash, path)),
          EntryType::File => self.upload_package_file(client, &path, &entry, options)?,
        }
      }
    }

    Ok(())
  }

  fn upload_package_file(
    &self,
    client: &Client,
    path: &Utf8Path,
    expected: &Entry,
    options: &Options,
  ) -> Result {
    let actual = options
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

    self.upload_body(client, expected.hash, file.into())?;

    Ok(())
  }
}
