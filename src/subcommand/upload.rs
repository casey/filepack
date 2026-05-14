use super::*;

// todo:
// - change address to URL
// - make sure I only have a single TLS crate and crypto provider in-tree

#[derive(Parser)]
pub(crate) struct Upload {
  address: String,
  file: Utf8PathBuf,
}

impl Upload {
  pub(crate) fn run(self) -> Result {
    let file = filesystem::read(&self.file)?;

    let client = reqwest::blocking::Client::new();

    client
      .put(self.address)
      .body(file)
      .send()
      .unwrap()
      .error_for_status()
      .unwrap();

    Ok(())
  }
}
