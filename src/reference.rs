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
pub struct Reference {
  pub commands: Command,
  pub schemas: BTreeMap<String, Schema>,
}

impl Reference {
  pub fn new() -> Self {
    let mut command = Arguments::command();

    command.build();

    let mut schemas = BTreeMap::new();

    String::schema(&mut schemas);
    Vec::<u8>::schema(&mut schemas);
    bool::schema(&mut schemas);
    i64::schema(&mut schemas);
    u64::schema(&mut schemas);

    Self {
      commands: Command::new(&mut command),
      schemas,
    }
  }
}
