use super::*;

// todo:
// - change address to URL
// - complain if file already exists?

#[derive(Parser)]
pub(crate) struct Download {
  address: String,
  hash: Hash,
  output: Utf8PathBuf,
}

impl Download {
  pub(crate) fn run(self) -> Result {
    let file = reqwest::blocking::Client::new()
      .get(self.address)
      .send()
      .unwrap()
      .error_for_status()
      .unwrap()
      .bytes()
      .unwrap();

    filesystem::write(&self.output, file).unwrap();

    Ok(())
  }
}
