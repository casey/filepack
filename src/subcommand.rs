use {
  super::*,
  clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
  },
};

mod contains;
mod create;
mod files;
mod fingerprint;
mod hash;
mod info;
mod key;
mod keygen;
mod languages;
mod lints;
mod man;
mod sign;
mod size;
mod verify;

const MANIFEST_PATH_HELP: &str = "\
  Load manifest from <PATH>. May be path to manifest, to directory containing manifest named \
  `filepack.json`, or omitted, in which case manifest named `filepack.json` in the current \
  directory is loaded.";

const TIME_HELP: &str = "Include current time in note";

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
  #[command(about = "Check if manifest contains file")]
  Contains(contains::Contains),
  #[command(about = "Create manifest")]
  Create(create::Create),
  #[command(about = "List manifest files")]
  Files(files::Files),
  #[command(about = "Print manifest fingerprint")]
  Fingerprint(fingerprint::Fingerprint),
  #[command(about = "Print file hash")]
  Hash(hash::Hash),
  #[command(about = "Print info")]
  Info,
  #[command(about = "Print public key")]
  Key(key::Key),
  #[command(about = "Generate key pair")]
  Keygen(keygen::Keygen),
  #[command(about = "List language codes")]
  Languages(languages::Languages),
  #[command(about = "List lint groups")]
  Lints,
  #[command(about = "Print man page")]
  Man,
  #[command(about = "Sign manifest")]
  Sign(sign::Sign),
  #[command(about = "Print manifest total file size")]
  Size(size::Size),
  #[command(about = "Verify manifest")]
  Verify(verify::Verify),
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Contains(contains) => contains.run(),
      Self::Create(create) => create.run(options),
      Self::Files(files) => files.run(),
      Self::Fingerprint(fingerprint) => fingerprint.run(),
      Self::Hash(hash) => hash.run(options),
      Self::Info => info::run(options),
      Self::Key(key) => key.run(options),
      Self::Languages(languages) => languages.run(),
      Self::Keygen(keygen) => keygen.run(options),
      Self::Lints => lints::run(),
      Self::Man => man::run(),
      Self::Sign(sign) => sign.run(options),
      Self::Size(size) => size.run(),
      Self::Verify(verify) => verify.run(options),
    }
  }
}
