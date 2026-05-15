use super::*;

// todo:
// - change address to URL

#[derive(Parser)]
pub(crate) struct Upload {
  server: Url,
  file: Utf8PathBuf,
}

impl Upload {
  pub(crate) fn run(self) -> Result {
    let file = filesystem::read(&self.file)?;

    let hash = Hash::bytes(&file);

    let response = reqwest::blocking::Client::new()
      .put(self.server.join(&hash.to_string()).unwrap())
      .body(file)
      .send()
      .unwrap();

    assert_eq!(response.status(), 400);

    Ok(())
  }
}
