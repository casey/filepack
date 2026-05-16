use super::*;

#[derive(Parser)]
pub(crate) struct Upload {
  #[arg(help = "Upload file at <PATH>", long, value_name = "PATH")]
  file: Utf8PathBuf,
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL")]
  server: Url,
}

impl Upload {
  pub(crate) fn run(self, options: Options) -> Result {
    let file = options
      .hash_file(&self.file)
      .context(error::FilesystemIo { path: &self.file })?;

    let url = self
      .server
      .join(&file.hash.to_string())
      .context(error::UrlParse)?;

    let file = filesystem::open(&self.file)?;

    let response = Client::new()
      .put(url.clone())
      .body(file)
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
}
