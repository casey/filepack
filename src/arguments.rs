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
