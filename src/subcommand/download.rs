use super::*;

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
      (Some(hash), None) => self.download_file(hash),
      (None, Some(hash)) => self.download_package(hash),
      (None, None) | (Some(_), Some(_)) => unreachable!(),
    }
  }

  pub(crate) fn download_file(self, hash: Hash) -> Result {
    ensure! {
      !filesystem::exists(&self.output)?,
      error::FileAlreadyExists { path: &self.output },
    }

    let url = self
      .server
      .join(&hash.to_string())
      .context(error::UrlParse)?;

    let mut response = Client::new()
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

    let output_directory = self
      .output
      .parent()
      .filter(|parent| !parent.as_str().is_empty())
      .unwrap_or(Utf8Path::new("."));

    let tempfile = transfer_tempfile(hash, output_directory).context(error::FilesystemIo {
      path: output_directory,
    })?;

    let mut writer = HashingWriter::new(tempfile);

    response
      .copy_to(&mut writer)
      .with_context(|_| error::ResponseBody { url: url.clone() })?;

    let (actual, tempfile) = writer.finalize();

    ensure! {
      actual == hash,
      error::DownloadHashMismatch { actual, expected: hash },
    }

    tempfile
      .persist_noclobber(&self.output)
      .map_err(|error| error.error)
      .context(error::FilesystemIo { path: &self.output })?;

    Ok(())
  }

  pub(crate) fn download_package(self, hash: Hash) -> Result {
    todo!()
  }
}
