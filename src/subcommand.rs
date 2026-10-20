use {
  super::*,
  clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
  },
  json::Io,
};

mod archive;
mod completions;
mod contains;
mod create;
mod delete;
mod download;
mod files;
mod fingerprint;
mod gc;
mod hash;
mod info;
mod key;
mod keygen;
mod languages;
mod lints;
mod man;
mod manifest;
mod metadata;
mod reference;
pub(crate) mod serve;
mod sign;
mod signatures;
mod size;
mod upload;
mod verify;

const MANIFEST_PATH_HELP: &str = "\
  Load manifest from <PATH>. May be path to manifest, to directory containing manifest named \
  `manifest.filepack`, or omitted, in which case manifest named `manifest.filepack` in the current \
  directory is loaded.";

const TIMESTAMP_HELP: &str = "Include current time in signature";

#[derive(EnumDiscriminants, Parser)]
#[strum_discriminants(
  name(Name),
  derive(EnumString),
  vis(pub(crate)),
  strum(serialize_all = "kebab-case")
)]
#[command(
  version,
  about = "Content-addressed packages",
  long_about = "Content-addressed packages - https://github.com/casey/filepack",
  styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default()))
]
pub(crate) enum Subcommand {
  #[command(about = "Create archive from JSON manifest")]
  Archive(archive::Archive),
  #[command(about = "Print shell completion script")]
  Completions(completions::Completions),
  #[command(about = "Check if manifest contains file")]
  Contains(contains::Contains),
  #[command(about = "Create manifest")]
  Create(create::Create),
  #[command(about = "Delete package from a filepack server")]
  Delete(delete::Delete),
  #[command(about = "Download package or file from a filepack server")]
  Download(download::Download),
  #[command(about = "List manifest files")]
  Files(files::Files),
  #[command(about = "Print package fingerprint")]
  Fingerprint(fingerprint::Fingerprint),
  #[command(about = "Delete unreferenced data on a filepack server")]
  Gc(gc::Gc),
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
  #[command(about = "Print manifest")]
  Manifest(manifest::Manifest),
  #[command(about = "Print metadata")]
  Metadata(metadata::Metadata),
  #[command(about = "Print reference")]
  Reference,
  #[command(about = "Start a filepack server")]
  Serve(serve::Serve),
  #[command(about = "Sign manifest")]
  Sign(sign::Sign),
  #[command(about = "List manifest signatures")]
  Signatures(signatures::Signatures),
  #[command(about = "Print manifest totals")]
  Size(size::Size),
  #[command(about = "Upload package or file to a filepack server")]
  Upload(upload::Upload),
  #[command(about = "Verify manifest")]
  Verify(verify::Verify),
}

impl Name {
  pub(crate) fn input(self, registry: &mut json::Registry) -> Option<String> {
    match self {
      Self::Archive => archive::Input::io_schema(registry),
      Self::Completions => completions::Input::io_schema(registry),
      Self::Contains => contains::Input::io_schema(registry),
      Self::Create => create::Input::io_schema(registry),
      Self::Delete => delete::Input::io_schema(registry),
      Self::Download => download::Input::io_schema(registry),
      Self::Files => files::Input::io_schema(registry),
      Self::Fingerprint => fingerprint::Input::io_schema(registry),
      Self::Gc => gc::Input::io_schema(registry),
      Self::Hash => hash::Input::io_schema(registry),
      Self::Info => info::Input::io_schema(registry),
      Self::Key => key::Input::io_schema(registry),
      Self::Keygen => keygen::Input::io_schema(registry),
      Self::Languages => languages::Input::io_schema(registry),
      Self::Lints => lints::Input::io_schema(registry),
      Self::Man => man::Input::io_schema(registry),
      Self::Manifest => manifest::Input::io_schema(registry),
      Self::Metadata => metadata::Input::io_schema(registry),
      Self::Reference => reference::Input::io_schema(registry),
      Self::Serve => serve::Input::io_schema(registry),
      Self::Sign => sign::Input::io_schema(registry),
      Self::Signatures => signatures::Input::io_schema(registry),
      Self::Size => size::Input::io_schema(registry),
      Self::Upload => upload::Input::io_schema(registry),
      Self::Verify => verify::Input::io_schema(registry),
    }
  }

  pub(crate) fn output(self, registry: &mut json::Registry) -> Option<String> {
    match self {
      Self::Archive => archive::Output::io_schema(registry),
      Self::Completions => completions::Output::io_schema(registry),
      Self::Contains => contains::Output::io_schema(registry),
      Self::Create => create::Output::io_schema(registry),
      Self::Delete => delete::Output::io_schema(registry),
      Self::Download => download::Output::io_schema(registry),
      Self::Files => files::Output::io_schema(registry),
      Self::Fingerprint => fingerprint::Output::io_schema(registry),
      Self::Gc => gc::Output::io_schema(registry),
      Self::Hash => hash::Output::io_schema(registry),
      Self::Info => info::Output::io_schema(registry),
      Self::Key => key::Output::io_schema(registry),
      Self::Keygen => keygen::Output::io_schema(registry),
      Self::Languages => languages::Output::io_schema(registry),
      Self::Lints => lints::Output::io_schema(registry),
      Self::Man => man::Output::io_schema(registry),
      Self::Manifest => manifest::Output::io_schema(registry),
      Self::Metadata => metadata::Output::io_schema(registry),
      Self::Reference => reference::Output::io_schema(registry),
      Self::Serve => serve::Output::io_schema(registry),
      Self::Sign => sign::Output::io_schema(registry),
      Self::Signatures => signatures::Output::io_schema(registry),
      Self::Size => size::Output::io_schema(registry),
      Self::Upload => upload::Output::io_schema(registry),
      Self::Verify => verify::Output::io_schema(registry),
    }
  }
}

impl Subcommand {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Archive(archive) => archive.run(),
      Self::Completions(completions) => completions.run(),
      Self::Contains(contains) => contains.run(options),
      Self::Create(create) => create.run(options),
      Self::Delete(delete) => delete.run(options),
      Self::Download(download) => download.run(options),
      Self::Files(files) => files.run(),
      Self::Fingerprint(fingerprint) => fingerprint.run(),
      Self::Gc(gc) => gc.run(options),
      Self::Hash(hash) => hash.run(options),
      Self::Info => info::run(options),
      Self::Key(key) => key.run(options),
      Self::Keygen(keygen) => keygen.run(options),
      Self::Languages(languages) => languages.run(),
      Self::Lints => lints::run(),
      Self::Man => man::run(),
      Self::Manifest(manifest) => manifest.run(),
      Self::Metadata(metadata) => metadata.run(),
      Self::Reference => reference::run(),
      Self::Serve(serve) => serve.run(options),
      Self::Sign(sign) => sign.run(options),
      Self::Signatures(signatures) => signatures.run(),
      Self::Size(size) => size.run(),
      Self::Upload(upload) => upload.run(options),
      Self::Verify(verify) => verify.run(options),
    }
  }
}
