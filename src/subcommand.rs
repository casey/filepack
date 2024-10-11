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
mod render;
mod sign;
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
  #[command(about = "Create manifest")]
  Create(create::Create),
  #[command(about = "Hash file")]
  Hash(hash::Hash),
  #[command(about = "Print master key")]
  Key,
  #[command(about = "Generate master key")]
  Keygen,
  #[command(about = "Print man page")]
  Man,
  #[command(about = "Render package as HTML")]
  Render(render::Render),
  #[command(about = "Add signature to manifest")]
  Sign(sign::Sign),
  #[command(about = "Verify manifest")]
  Verify(verify::Verify),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Create(create) => create.run(options),
      Self::Hash(hash) => hash.run(options),
      Self::Key => key::run(options),
      Self::Keygen => keygen::run(options),
      Self::Man => man::run(),
      Self::Render(render) => render.run(),
      Self::Sign(sign) => sign.run(options),
      Self::Verify(verify) => verify.run(options),
    }
  }
}
