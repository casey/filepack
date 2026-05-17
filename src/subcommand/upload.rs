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
      (Some(path), None) => self.upload_file(&path, &options),
      (None, Some(path)) => self.upload_package(&path, options),
      (None, None) | (Some(_), Some(_)) => unreachable!(),
    }
  }

  fn upload_body(&self, hash: Hash, body: Body) -> Result {
    let url = self
      .server
      .join(&hash.to_string())
      .context(error::UrlParse)?;

    let response = Client::new()
      .put(url.clone())
      .body(body)
      .send()
      .with_context(|_| error::Request { url: url.clone() })?;

    if !response.status().is_success() {
      return Err(
        error::ResponseStatus {
          status: response.status(),
          url: url.clone(),
          body: response.text().context(error::ResponseBody { url })?,
        }
        .build(),
      );
    }

    Ok(())
  }

  fn upload_file(&self, path: &Utf8Path, options: &Options) -> Result {
    let hash = options
      .hash_file(&path)
      .context(error::FilesystemIo { path })?
      .hash;

    let file = filesystem::open(&path)?;

    self.upload_body(hash, file.into())?;

    Ok(())
  }

  fn upload_package(&self, path: &Utf8Path, options: Options) -> Result {
    let archive = Archive::load_with_path(path, path)?;

    let mut directories = vec![(
      archive.fingerprint().into(),
      path.parent().unwrap().to_owned(),
    )];

    while let Some((hash, path)) = directories.pop() {
      let directory = archive.files.get(&hash).unwrap();

      self.upload_body(hash, directory.clone().into())?;

      let directory = Directory::decode_from_slice(directory).unwrap();

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry.ty {
          EntryType::Directory => directories.push((hash, path)),
          EntryType::File => self.upload_file(&path, &options)?,
        }
      }
    }

    Ok(())
  }
}
