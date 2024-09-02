use {
  super::*,
  clap::builder::{
    styling::{AnsiColor, Effects},
    Styles,
  },
};

mod create;
mod verify;

#[derive(Parser)]
#[command(
  version,
  styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default()))
]
pub(crate) enum Subcommand {
  Create { root: Utf8PathBuf },
  Verify { root: Utf8PathBuf },
}

impl Subcommand {
  pub(crate) fn run(self) -> Result {
    match self {
      Self::Create { root } => create::run(&root),
      Self::Verify { root } => verify::run(&root),
    }
  }
}
