use {super::*, reqwest::blocking::Body};

#[derive(Parser)]
pub(crate) struct Upload {
  #[arg(help = "Upload file instead of package", long)]
  file: bool,
  #[arg(help = "Upload <PATH>", value_name = "PATH")]
  input: Utf8PathBuf,
  #[arg(
    help = "Authenticate upload with private key <KEY>",
    long,
    value_name = "KEY"
  )]
  key: Option<KeyName>,
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL", value_parser = parse_server_url)]
  server: Url,
}

impl Upload {
  pub(crate) fn run(self, options: Options) -> Result {
    let token = if let Some(name) = &self.key {
      ensure!(
        self.server.scheme() == "https",
        error::InsecureUploadKey {
          server: self.server.clone(),
        },
      );

      let keychain = Keychain::load(&options)?;

      let private_key = PrivateKey::load(&keychain.path.join(name.private_key_filename()))?;

      let host = self
        .server
        .host_str()
        .context(error::ServerHost {
          server: self.server.clone(),
        })?
        .to_owned();

      Some(jwt::encode(&private_key, &host)?)
    } else {
      None
    };

    if self.file {
      self.upload_file(&self.input, &options, token.as_deref())
    } else {
      self.upload_package(&self.input, &options, token.as_deref())
    }
  }

  fn upload_body(&self, hash: Hash, body: Body, token: Option<&str>) -> Result {
    let url = self.server.join(&hash.to_string()).unwrap();
    let mut request = Client::new().put(url).body(body);
    if let Some(token) = token {
      request = request.bearer_auth(token);
    }
    request.send().check_status()?;
    Ok(())
  }

  fn upload_file(&self, path: &Utf8Path, options: &Options, token: Option<&str>) -> Result {
    let hash = options
      .hash_file(path)
      .context(error::FilesystemIo { path })?
      .hash;

    let file = filesystem::open(path)?;

    self.upload_body(hash, file.into(), token)?;

    Ok(())
  }

  fn upload_package(
    &self,
    archive_path: &Utf8Path,
    options: &Options,
    token: Option<&str>,
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

      self.upload_body(hash, cbor.to_vec().into(), token)?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry.ty {
          EntryType::Directory => directories.push((entry.hash, path)),
          EntryType::File => self.upload_package_file(&path, &entry, options, token)?,
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
    token: Option<&str>,
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

    self.upload_body(expected.hash, file.into(), token)?;

    Ok(())
  }
}
