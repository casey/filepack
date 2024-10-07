use {
  self::{
    arguments::Arguments, display_path::DisplayPath, display_secret::DisplaySecret, entry::Entry,
    error::Error, github_release::GithubRelease, hash::Hash, lint::Lint, lint_group::LintGroup,
    list::List, manifest::Manifest, metadata::Metadata, options::Options,
    owo_colorize_ext::OwoColorizeExt, private_key::PrivateKey, public_key::PublicKey,
    relative_path::RelativePath, signature::Signature, signature_error::SignatureError,
    style::Style, subcommand::Subcommand, template::Template,
  },
  blake3::Hasher,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
  owo_colors::Styled,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  snafu::{ensure, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    array::TryFromSliceError,
    backtrace::{Backtrace, BacktraceStatus},
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    env,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, IsTerminal},
    path::{Path, PathBuf},
    process,
    str::{self, FromStr},
  },
  walkdir::WalkDir,
};

#[cfg(test)]
use assert_fs::TempDir;

mod arguments;
mod display_path;
mod display_secret;
mod entry;
mod error;
mod github_release;
mod hash;
mod lint;
mod lint_group;
mod list;
mod manifest;
mod metadata;
mod options;
mod owo_colorize_ext;
mod private_key;
mod progress_bar;
mod public_key;
mod relative_path;
mod signature;
mod signature_error;
mod style;
mod subcommand;
mod template;

type Result<T = (), E = Error> = std::result::Result<T, E>;

const MASTER_PRIVATE_KEY: &str = "master.private";
const MASTER_PUBLIC_KEY: &str = "master.public";
const SIGNATURES: &str = "signatures";

fn current_dir() -> Result<PathBuf> {
  env::current_dir().context(error::CurrentDir)
}

fn main() {
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

    if let Some(backtrace) = err.backtrace() {
      if backtrace.status() == BacktraceStatus::Captured {
        eprintln!();
        eprintln!("backtrace:");
        eprintln!("{backtrace}");
      }
    }

    process::exit(1);
  }
}
