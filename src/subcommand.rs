use {
  super::*,
  clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
  },
};

const MANIFEST_PATH_HELP: &str = "\
  Load manifest from <PATH>. May be path to manifest, to directory containing manifest named \
  `filepack.json`, or omitted, in which case manifest named `filepack.json` in the current \
  directory is loaded.";

mod create;
mod files;
mod fingerprint;
mod hash;
mod key;
mod keygen;
mod man;
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
  #[command(about = "List manifest files")]
  Files(files::Files),
  #[command(about = "Print manifest fingerprint")]
  Fingerprint(fingerprint::Fingerprint),
  #[command(about = "Print file hash")]
  Hash(hash::Hash),
  #[command(about = "Print master public key")]
  Key,
  #[command(about = "Generate master key")]
  Keygen,
  #[command(about = "Print man page")]
  Man,
  #[command(about = "Sign manifest")]
  Sign(sign::Sign),
  #[command(about = "Verify manifest")]
  Verify(verify::Verify),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Create(create) => create.run(options),
      Self::Files(files) => files.run(),
      Self::Fingerprint(fingerprint) => fingerprint.run(),
      Self::Hash(hash) => hash.run(options),
      Self::Key => key::run(options),
      Self::Keygen => keygen::run(options),
      Self::Man => man::run(),
      Self::Sign(sign) => sign.run(options),
      Self::Verify(verify) => verify.run(options),
    }
  }
}
