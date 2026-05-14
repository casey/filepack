use super::*;

// todo:
// - change address to URL
// - accept domain without HTTP or HTTPS
// - make sure I only have a single TLS crate and crypto provider in-tree

#[derive(Parser)]
pub(crate) struct Upload {
  address: Url,
  file: Utf8PathBuf,
}

impl Upload {
  pub(crate) fn run(self) -> Result {
    let file = filesystem::read(&self.file)?;

    let client = reqwest::blocking::Client::new();

    let hash = Hash::bytes(&file);

    client
      .put(self.address.join(&hash.to_string()).unwrap())
      .body(file)
      .send()
      .unwrap()
      .error_for_status()
      .unwrap();

    Ok(())
  }
}
