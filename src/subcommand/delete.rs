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
    Client::new(&options, self.server.clone(), self.auth.as_ref())?.delete_package(self.fingerprint)
  }
}
