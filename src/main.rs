use {
  self::{
    arguments::Arguments, display_path::DisplayPath, entry::Entry, error::Error, hash::Hash,
    lint::Lint, list::List, manifest::Manifest, options::Options, relative_path::RelativePath,
    subcommand::Subcommand,
  },
  blake3::Hasher,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::Parser,
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  snafu::{ensure, ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    collections::{BTreeMap, HashMap},
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
mod display_path;
mod entry;
mod error;
mod hash;
mod lint;
mod list;
mod manifest;
mod options;
mod progress_bar;
mod relative_path;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(err) = Arguments::parse().run() {
    eprintln!("error: {err}");

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
