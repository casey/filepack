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
    help = "Delete package number <NUMBER>",
    value_name = "NUMBER"
  )]
  number: Option<u64>,
  #[arg(help = "Delete from server at <URL>", long, value_name = "URL")]
  server: ServerUrl,
}

impl Delete {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = Client::new(&options, self.server.clone(), self.auth.as_ref())?;

    if self.all {
      for number in client.numbers()? {
        client.delete_number(number)?;
      }

      Ok(())
    } else {
      client.delete_number(self.number.unwrap())
    }
  }
}
