use {super::*, clap::CommandFactory};

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
  pub help: String,
  pub name: String,
  pub subcommands: BTreeMap<String, Command>,
}

impl Command {
  fn new(command: &mut clap::Command) -> Self {
    Self {
      help: command.render_long_help().to_string(),
      name: command.get_name().into(),
      subcommands: command
        .get_subcommands_mut()
        .map(|command| (command.get_name().to_string(), Command::new(command)))
        .collect(),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
  pub commands: Command,
}

impl Schema {
  pub fn new() -> Self {
    let mut command = Arguments::command();

    command.build();

    Self {
      commands: Command::new(&mut command),
    }
  }
}
