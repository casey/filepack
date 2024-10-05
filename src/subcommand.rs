use {
  super::*,
  clap::builder::{
    styling::{AnsiColor, Effects},
    Styles,
  },
};

mod create;
mod hash;
mod key;
mod keygen;
mod man;
mod verify;

#[derive(Parser)]
#[command(
  version,
  about = "🗄️ file verification utility - https://github.com/casey/filepack",
  styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default()))
]
pub(crate) enum Subcommand {
  #[command(about = "create manifest")]
  Create(create::Create),
  #[command(about = "hash single file")]
  Hash(hash::Hash),
  #[command(about = "Generate new key pair")]
  Keygen,
  #[command(about = "Print master key")]
  Key,
  #[command(about = "print man page")]
  Man,
  #[command(about = "verify directory against manifest")]
  Verify(verify::Verify),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Create(create) => create.run(options),
      Self::Hash(hash) => hash.run(options),
      Self::Keygen => keygen::run(options),
      Self::Key => key::run(options),
      Self::Man => man::run(),
      Self::Verify(verify) => verify.run(options),
    }
  }
}
