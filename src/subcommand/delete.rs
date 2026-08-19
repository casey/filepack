use super::*;

#[derive(Parser)]
#[command(group = ArgGroup::new("target").required(true))]
pub(crate) struct Delete {
  #[arg(group = "target", help = "Delete all packages", long)]
  all: bool,
  #[arg(help = "Authenticate with key <KEY>", long, value_name = "KEY")]
  auth: Option<KeyName>,
  #[arg(
    group = "target",
    help = "Delete package with <FINGERPRINT>",
    value_name = "FINGERPRINT"
  )]
  fingerprint: Option<Fingerprint>,
  #[arg(help = "Delete from server at <URL>", long, value_name = "URL", value_parser = CheckedUrl::check)]
  server: Url,
}

impl Delete {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = Client::new(&options, self.server.clone(), self.auth.as_ref())?;

    if self.all {
      for fingerprint in client.packages()? {
        client.delete_package(fingerprint)?;
      }

      Ok(())
    } else {
      client.delete_package(self.fingerprint.unwrap())
    }
  }
}
