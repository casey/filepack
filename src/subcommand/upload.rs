use {super::*, reqwest::blocking::Body};

#[derive(Parser)]
#[command(group(
  ArgGroup::new("input")
    .required(true)
    .args(["file", "package"]),
))]
pub(crate) struct Upload {
  #[arg(help = "Upload file at <PATH>", long, value_name = "PATH")]
  file: Option<Utf8PathBuf>,
  #[arg(help = "Upload package at <PATH>", long, value_name = "PATH")]
  package: Option<Utf8PathBuf>,
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL")]
  server: Url,
}

impl Upload {
  pub(crate) fn run(self, options: Options) -> Result {
    match (&self.file, &self.package) {
      (Some(path), None) => self.upload_file(path, &options),
      (None, Some(path)) => self.upload_package(path, options),
      (None, None) | (Some(_), Some(_)) => unreachable!(),
    }
  }

  fn upload_body(&self, hash: Hash, body: Body) -> Result {
    let url = self
      .server
      .join(&hash.to_string())
      .context(error::UrlParse)?;

    Client::new().put(url).body(body).send().check_status()?;

    Ok(())
  }

  fn upload_file(&self, path: &Utf8Path, options: &Options) -> Result {
    let hash = options
      .hash_file(path)
      .context(error::FilesystemIo { path })?
      .hash;

    let file = filesystem::open(path)?;

    self.upload_body(hash, file.into())?;

    Ok(())
  }

  fn upload_package(&self, archive_path: &Utf8Path, options: Options) -> Result {
    let archive = Archive::load_with_path(archive_path, archive_path)?;

    let fingerprint = archive
      .fingerprint()
      .context(error::UnarchiveManifest { path: archive_path })?;

    let mut directories = vec![(
      fingerprint.into(),
      archive_path.parent().unwrap().to_owned(),
    )];

    while let Some((hash, path)) = directories.pop() {
      let cbor = archive
        .file(hash)
        .context(error::UnarchiveManifest { path: archive_path })?;

      let directory = Directory::decode_from_slice(cbor)
        .context(archive_error::DirectoryDecode)
        .context(error::UnarchiveManifest { path: archive_path })?;

      self.upload_body(hash, cbor.to_vec().into())?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry.ty {
          EntryType::Directory => directories.push((entry.hash, path)),
          EntryType::File => self.upload_package_file(&path, &entry, &options)?,
        }
      }
    }

    Ok(())
  }

  fn upload_package_file(&self, path: &Utf8Path, expected: &Entry, options: &Options) -> Result {
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

    self.upload_body(expected.hash, file.into())?;

    Ok(())
  }
}
