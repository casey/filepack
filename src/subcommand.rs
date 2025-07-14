use {
  super::*,
  clap::builder::{
    styling::{AnsiColor, Effects},
    Styles,
  },
};

mod archive;
mod create;
mod fingerprint;
mod hash;
mod key;
mod keygen;
mod man;
mod render;
mod server;
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
  #[command(about = "Create archive")]
  Archive(archive::Archive),
  #[command(about = "Create manifest")]
  Create(create::Create),
  #[command(about = "Print manifest fingerprint")]
  Fingerprint(fingerprint::Fingerprint),
  #[command(about = "Print file hash")]
  Hash(hash::Hash),
  #[command(about = "Print master key")]
  Key,
  #[command(about = "Generate master key")]
  Keygen,
  #[command(about = "Print man page")]
  Man,
  #[command(about = "Render manifest")]
  Render(render::Render),
  #[command(about = "Run the server")]
  Server(server::Server),
  #[command(about = "Sign manifest")]
  Sign(sign::Sign),
  #[command(about = "Verify manifest")]
  Verify(verify::Verify),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Archive(archive) => archive.run(),
      Self::Create(create) => create.run(options),
      Self::Fingerprint(fingerprint) => fingerprint.run(),
      Self::Hash(hash) => hash.run(options),
      Self::Key => key::run(options),
      Self::Keygen => keygen::run(options),
      Self::Man => man::run(),
      Self::Render(render) => render.run(),
      Self::Server(server) => server.run(),
      Self::Sign(sign) => sign.run(options),
      Self::Verify(verify) => verify.run(options),
    }
  }
}
