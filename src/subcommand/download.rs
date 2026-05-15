use super::*;

#[derive(Parser)]
pub(crate) struct Download {
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL")]
  server: Url,
  #[arg(help = "Download file with <HASH>", long)]
  hash: Hash,
  #[arg(help = "Download to <PATH>", long, value_name = "PATH")]
  output: Utf8PathBuf,
}

impl Download {
  pub(crate) fn run(self) -> Result {
    let response = reqwest::blocking::Client::new()
      .get(self.server.join(&self.hash.to_string()).unwrap())
      .send()
      .unwrap();

    assert_eq!(response.status(), 200);

    let file = response.bytes().unwrap();

    filesystem::write_new(&self.output, file)?;

    Ok(())
  }
}
