use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
  pub about: String,
  pub help: String,
  pub input: Option<String>,
  pub name: String,
  pub output: Option<String>,
  pub subcommands: BTreeMap<String, Command>,
}

impl Command {
  pub(crate) fn new(
    command: &mut clap::Command,
    name: Option<subcommand::Name>,
    registry: &mut json::Registry,
  ) -> Self {
    Self {
      about: command.get_about().unwrap().to_string(),
      help: command.render_long_help().to_string(),
      input: name.and_then(|name| name.input(registry)),
      name: command.get_name().into(),
      output: name.and_then(|name| name.output(registry)),
      subcommands: command
        .get_subcommands_mut()
        .filter(|command| command.get_name() != "help")
        .map(|command| {
          let name = command.get_name().parse::<subcommand::Name>().unwrap();
          (
            command.get_name().to_string(),
            Command::new(command, Some(name), registry),
          )
        })
        .collect(),
    }
  }
}
