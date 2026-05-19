use {super::*, reqwest::blocking::Body, url::Host};

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
  pub(crate) fn run(self, options: Options) -> Result {
    let key = if let Some(name) = &self.auth {
      let loopback = match self.server.host().unwrap() {
        Host::Domain(domain) => domain == "localhost",
        Host::Ipv4(addr) => addr.is_loopback(),
        Host::Ipv6(addr) => addr.is_loopback(),
      };

      ensure!(
        self.server.scheme() == "https" || loopback,
        error::AuthenticationOverHttp {
          server: self.server.clone(),
        },
      );

      let keychain = Keychain::load(&options)?;

      Some(PrivateKey::load(
        &keychain.path.join(name.private_key_filename()),
      )?)
    } else {
      None
    };

    if self.file {
      self.upload_file(&self.input, &options, key.as_ref())
    } else {
      self.upload_package(&self.input, &options, key.as_ref())
    }
  }

  fn upload_body(&self, hash: Hash, body: Body, key: Option<&PrivateKey>) -> Result {
    let url = self.server.join(&format!("file/{hash}")).unwrap();
    let mut request = Client::new().put(url).body(body);
    if let Some(key) = key {
      let host = self.server.host_str().unwrap().to_owned();
      request = request.bearer_auth(Token::encode(key, &host)?);
    }
    request.send().check_status()?;
    Ok(())
  }

  fn upload_file(&self, path: &Utf8Path, options: &Options, key: Option<&PrivateKey>) -> Result {
    let hash = options
      .hash_file(path)
      .context(error::FilesystemIo { path })?
      .hash;

    let file = filesystem::open(path)?;

    self.upload_body(hash, file.into(), key)?;

    Ok(())
  }

  fn upload_package(
    &self,
    archive_path: &Utf8Path,
    options: &Options,
    key: Option<&PrivateKey>,
  ) -> Result {
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

      self.upload_body(hash, cbor.to_vec().into(), key)?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry.ty {
          EntryType::Directory => directories.push((entry.hash, path)),
          EntryType::File => self.upload_package_file(&path, &entry, options, key)?,
        }
      }
    }

    Ok(())
  }

  fn upload_package_file(
    &self,
    path: &Utf8Path,
    expected: &Entry,
    options: &Options,
    key: Option<&PrivateKey>,
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

    self.upload_body(expected.hash, file.into(), key)?;

    Ok(())
  }
}
