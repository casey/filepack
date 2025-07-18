use {
  self::{
    archive::Archive, archive_error::ArchiveError, arguments::Arguments, bytes::Bytes,
    display_path::DisplayPath, display_secret::DisplaySecret, entry::Entry, error::Error,
    hash::Hash, into_u64::IntoU64, lint::Lint, lint_group::LintGroup, list::List,
    manifest::Manifest, metadata::Metadata, options::Options, owo_colorize_ext::OwoColorizeExt,
    private_key::PrivateKey, public_key::PublicKey, relative_path::RelativePath,
    signature::Signature, signature_error::SignatureError, style::Style, subcommand::Subcommand,
    template::Template, utf8_path_ext::Utf8PathExt,
  },
  blake3::Hasher,
  boilerplate::Boilerplate,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  html_escaper::{Escape, Trusted},
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
  owo_colors::Styled,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  serde_with::{DeserializeFromStr, SerializeDisplay},
  snafu::{ensure, ErrorCompat, IntoError, OptionExt, ResultExt, Snafu},
  std::{
    array::TryFromSliceError,
    backtrace::{Backtrace, BacktraceStatus},
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    env,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, BufReader, BufWriter, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process,
    str::{self, FromStr},
    sync::Arc,
  },
  tokio::runtime::Runtime,
  walkdir::WalkDir,
};

#[cfg(test)]
use {
  assert_fs::{
    fixture::{FileWriteBin, FileWriteStr, PathChild, PathCreateDir},
    TempDir,
  },
  std::ffi::OsString,
};

#[cfg(test)]
macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}

#[cfg(test)]
macro_rules! command {
  ( $($argument:expr),* $(,)?) => {
    {
      #![allow(clippy::vec_init_then_push)]

      let mut arguments = Vec::<OsString>::new();

      arguments.push("filepack".into());

      arguments.push("--quiet".into());

      $(
        arguments.push($argument.into());
      )*

      Arguments::try_parse_from(arguments).unwrap().run().unwrap();
    }
  };
}

mod archive;
mod archive_error;
mod arguments;
mod bytes;
mod display_path;
mod display_secret;
mod entry;
mod error;
mod filesystem;
mod hash;
mod into_u64;
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
mod server_error;
mod signature;
mod signature_error;
mod style;
mod subcommand;
mod template;
mod utf8_path_ext;

type Result<T = (), E = Error> = std::result::Result<T, E>;

const MASTER_PRIVATE_KEY: &str = "master.private";
const MASTER_PUBLIC_KEY: &str = "master.public";

fn current_dir() -> Result<Utf8PathBuf> {
  Utf8PathBuf::from_path_buf(env::current_dir().context(error::CurrentDir)?)
    .map_err(|path| error::PathUnicode { path }.build())
}

fn decode_path(path: &Path) -> Result<&Utf8Path> {
  Utf8Path::from_path(path).context(error::PathUnicode { path })
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
