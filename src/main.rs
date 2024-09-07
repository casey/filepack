use {
  self::{
    arguments::Arguments, entry::Entry, error::Error, hash::Hash, lint::Lint, list::List,
    manifest::Manifest, options::Options, relative_path::RelativePath, subcommand::Subcommand,
  },
  blake3::Hasher,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::Parser,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  snafu::{ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    collections::HashMap,
    env,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process,
    str::{self, FromStr},
  },
  walkdir::WalkDir,
};

mod arguments;
mod entry;
mod error;
mod hash;
mod lint;
mod list;
mod manifest;
mod options;
mod relative_path;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(err) = Arguments::parse().run() {
    eprintln!("error: {err}");

    for (i, err) in err.iter_chain().skip(1).enumerate() {
      if i == 0 {
        eprintln!();
        eprintln!("because:");
      }

      eprintln!("- {err}");
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
