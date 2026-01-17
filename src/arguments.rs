use {
  super::*,
  clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
  },
};

#[derive(Parser)]
#[command(
  version,
  styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default()))
]
pub(crate) struct Arguments {
  #[command(flatten)]
  options: Options,
  #[command(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result {
    self.subcommand.run(self.options)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn commands_in_readme_are_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```shell(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      let command = capture[1].split('|').next().unwrap();
      assert!(
        Arguments::try_parse_from(command.split_whitespace()).is_ok(),
        "bad filepack command in readme: {command}",
      );
    }
  }
}
