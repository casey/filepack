use super::*;

// todo:
// - complain if file already exists?

#[derive(Parser)]
pub(crate) struct Download {
  server: Url,
  hash: Hash,
  output: Utf8PathBuf,
}

impl Download {
  pub(crate) fn run(self) -> Result {
    let response = reqwest::blocking::Client::new()
      .get(self.server.join(&self.hash.to_string()).unwrap())
      .send()
      .unwrap();

    assert_eq!(response.status(), 400);

    let file = response.bytes().unwrap();

    filesystem::write_new(&self.output, file)?;

    Ok(())
  }
}
