use {
  self::{
    arguments::Arguments, component::Component, context::Context, context_hasher::ContextHasher,
    directory::Directory, display_path::DisplayPath, display_secret::DisplaySecret,
    entries::Entries, entry::Entry, lint::Lint, lint_group::LintGroup, message::Message,
    metadata::Metadata, options::Options, owo_colorize_ext::OwoColorizeExt, path_error::PathError,
    private_key::PrivateKey, signature_error::SignatureError, style::Style, subcommand::Subcommand,
    template::Template, utf8_path_ext::Utf8PathExt,
  },
  blake3::Hasher,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
  owo_colors::Styled,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  serde_with::{DeserializeFromStr, SerializeDisplay},
  snafu::{ErrorCompat, OptionExt, ResultExt, Snafu, ensure},
  std::{
    array::TryFromSliceError,
    backtrace::{Backtrace, BacktraceStatus},
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    env,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, IsTerminal},
    iter,
    num::NonZeroU64,
    path::{Path, PathBuf},
    process,
    str::{self, FromStr},
  },
  strum::IntoStaticStr,
  usized::IntoU64,
  walkdir::WalkDir,
};

pub use self::{
  error::Error, file::File, hash::Hash, manifest::Manifest, public_key::PublicKey,
  relative_path::RelativePath, signature::Signature,
};

#[cfg(test)]
use assert_fs::TempDir;

mod arguments;
mod component;
mod context;
mod context_hasher;
mod directory;
mod display_path;
mod display_secret;
mod entries;
mod entry;
mod error;
mod file;
mod filesystem;
mod hash;
mod lint;
mod lint_group;
mod manifest;
mod message;
mod metadata;
mod options;
mod owo_colorize_ext;
mod path_error;
mod private_key;
mod progress_bar;
mod public_key;
mod relative_path;
mod signature;
mod signature_error;
mod style;
mod subcommand;
mod template;
mod utf8_path_ext;

const MASTER_PRIVATE_KEY: &str = "master.private";
const MASTER_PUBLIC_KEY: &str = "master.public";
const SEPARATORS: [char; 2] = ['/', '\\'];

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn current_dir() -> Result<Utf8PathBuf> {
  Utf8PathBuf::from_path_buf(env::current_dir().context(error::CurrentDir)?)
    .map_err(|path| error::PathUnicode { path }.build())
}

fn decode_path(path: &Path) -> Result<&Utf8Path> {
  Utf8Path::from_path(path).context(error::PathUnicode { path })
}

pub fn run() {
  if let Err(err) = Arguments::parse().run() {
    let style = Style::stderr();
    eprintln!(
      "{}: {}",
      "error".style(style.error()),
      err.style(style.message()),
    );

    let causes = err.iter_chain().skip(1).count();

    for (i, err) in err.iter_chain().skip(1).enumerate() {
      eprintln!("       {}─ {err}", if i < causes - 1 { '├' } else { '└' });
    }

    if let Some(backtrace) = err.backtrace()
      && backtrace.status() == BacktraceStatus::Captured
    {
      eprintln!();
      eprintln!("backtrace:");
      eprintln!("{backtrace}");
    }

    process::exit(1);
  }
}
