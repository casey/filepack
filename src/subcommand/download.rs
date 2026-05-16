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
    let url = self
      .server
      .join(&self.hash.to_string())
      .context(error::UrlParse)?;

    let response = Client::new()
      .get(url.clone())
      .send()
      .with_context(|_| error::Request { url: url.clone() })?;

    ensure! {
      response.status().is_success(),
      error::ResponseStatus { status: response.status(), url: url.clone() }
    }

    let file = response.bytes().context(error::ResponseBody { url })?;

    let actual = Hash::bytes(&file);

    ensure! {
      actual == self.hash,
      error::DownloadHashMismatch { actual, expected: self.hash },
    }

    filesystem::write_new(&self.output, file)?;

    Ok(())
  }
}
