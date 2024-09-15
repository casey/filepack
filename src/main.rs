use {
  self::{
    arguments::Arguments, display_path::DisplayPath, entry::Entry, error::Error, hash::Hash,
    lint::Lint, lint_group::LintGroup, list::List, manifest::Manifest, options::Options,
    owo_colorize_ext::OwoColorizeExt, relative_path::RelativePath, style::Style,
    subcommand::Subcommand,
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
    backtrace::{Backtrace, BacktraceStatus},
    collections::{BTreeMap, HashMap},
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

mod arguments;
mod display_path;
mod entry;
mod error;
mod hash;
mod lint;
mod lint_group;
mod list;
mod manifest;
mod options;
mod owo_colorize_ext;
mod progress_bar;
mod relative_path;
mod style;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

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
