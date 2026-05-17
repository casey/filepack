use {super::*, reqwest::blocking::Response};

#[derive(Parser)]
#[command(group(
  ArgGroup::new("input")
    .required(true)
    .args(["file", "package"]),
))]
pub(crate) struct Download {
  #[arg(help = "Download file with <HASH>", long)]
  file: Option<Hash>,
  #[arg(help = "Download to <PATH>", long, value_name = "PATH")]
  output: Utf8PathBuf,
  #[arg(help = "Download package with <HASH>", long)]
  package: Option<Hash>,
  #[arg(help = "Download from server at <URL>", long, value_name = "URL")]
  server: Url,
}

impl Download {
  pub(crate) fn run(self) -> Result {
    match (self.file, self.package) {
      (Some(hash), None) => self.download_file(hash, &self.output),
      (None, Some(hash)) => self.download_package(hash),
      (None, None) | (Some(_), Some(_)) => unreachable!(),
    }
  }

  pub(crate) fn download_file(&self, hash: Hash, path: &Utf8Path) -> Result {
    ensure! {
      !filesystem::exists(path)?,
      error::FileAlreadyExists { path },
    }

    let mut response = self.get_file(hash)?;

    let output_directory = self
      .output
      .parent()
      .filter(|parent| !parent.as_str().is_empty())
      .unwrap_or(Utf8Path::new("."));

    let tempfile = transfer_tempfile(hash, output_directory).context(error::FilesystemIo {
      path: output_directory,
    })?;

    let mut writer = HashingWriter::new(tempfile);

    let url = self
      .server
      .join(&hash.to_string())
      .context(error::UrlParse)?;

    response
      .copy_to(&mut writer)
      .with_context(|_| error::ResponseBody { url: url.clone() })?;

    let (actual, tempfile) = writer.finalize();

    ensure! {
      actual == hash,
      error::DownloadHashMismatch { actual, expected: hash },
    }

    tempfile
      .persist_noclobber(path)
      .map_err(|error| error.error)
      .context(error::FilesystemIo { path })?;

    Ok(())
  }

  pub(crate) fn download_package(self, root: Hash) -> Result {
    let mut directories = vec![(root, self.output.clone())];

    let mut files = BTreeMap::new();

    while let Some((hash, path)) = directories.pop() {
      let url = self
        .server
        .join(&hash.to_string())
        .context(error::UrlParse)?;

      let response = self.get_file(hash)?;

      let cbor = response.bytes().context(error::ResponseBody { url })?;

      let directory = Directory::decode_from_slice(&cbor).unwrap();

      files.insert(hash, cbor);

      filesystem::create_dir_all(&path)?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry.ty {
          EntryType::Directory => directories.push((hash, path)),
          EntryType::File => self.download_file(hash, &path)?,
        }
      }
    }

    Ok(())
  }

  fn get_file(&self, hash: Hash) -> Result<Response> {
    let url = self
      .server
      .join(&hash.to_string())
      .context(error::UrlParse)?;

    let response = Client::new()
      .get(url.clone())
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

    Ok(response)
  }
}
