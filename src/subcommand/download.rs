use super::*;

#[derive(Parser)]
pub(crate) struct Download {
  #[arg(help = "Download file with <HASH>", long)]
  hash: Hash,
  #[arg(help = "Download to <PATH>", long, value_name = "PATH")]
  output: Utf8PathBuf,
  #[arg(help = "Download from server at <URL>", long, value_name = "URL")]
  server: Url,
}

impl Download {
  pub(crate) fn run(self) -> Result {
    ensure! {
      !filesystem::exists(&self.output)?,
      error::FileAlreadyExists { path: &self.output },
    }

    let url = self
      .server
      .join(&self.hash.to_string())
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

    let tempfile = tempfile::Builder::new()
      .prefix(&format!("{}-", self.hash))
      .suffix(".incomplete")
      .tempfile_in(output_directory)
      .context(error::FilesystemIo {
        path: output_directory,
      })?;

    let mut writer = HashingWriter::new(tempfile);

    response
      .copy_to(&mut writer)
      .with_context(|_| error::ResponseBody { url: url.clone() })?;

    let (actual, tempfile) = writer.finalize();

    ensure! {
      actual == self.hash,
      error::DownloadHashMismatch { actual, expected: self.hash },
    }

    tempfile
      .persist_noclobber(&self.output)
      .map_err(|error| error.error)
      .context(error::FilesystemIo { path: &self.output })?;

    Ok(())
  }
}
