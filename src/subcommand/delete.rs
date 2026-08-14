use super::*;

#[derive(Parser)]
pub(crate) struct Delete {
  #[arg(help = "Authenticate with key <KEY>", long, value_name = "KEY")]
  auth: Option<KeyName>,
  #[arg(help = "Delete package with <FINGERPRINT>", value_name = "FINGERPRINT")]
  fingerprint: Fingerprint,
  #[arg(help = "Delete from server at <URL>", long, value_name = "URL", value_parser = CheckedUrl::check)]
  server: Url,
}

impl Delete {
  pub(crate) fn run(self, options: Options) -> Result {
    let key = load_auth_key(&options, &self.server, self.auth.as_ref())?;

    let url = self
      .server
      .join(&format!("api/package/{}", self.fingerprint))
      .unwrap();

    let request = client()?.delete(url);

    request_with_token(request, &self.server, key.as_ref())?
      .send()
      .check_status()?;

    Ok(())
  }
}
