use {
  self::{error::Error, filepack::Filepack, hash::Hash, subcommand::Subcommand},
  blake3::Hasher,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::Parser,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  snafu::{ErrorCompat, OptionExt, ResultExt, Snafu},
  std::{
    backtrace::BacktraceStatus,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io,
    path::PathBuf,
  },
  walkdir::WalkDir,
};

mod error;
mod filepack;
mod hash;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

fn main() {
  if let Err(err) = Subcommand::parse().run() {
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
  }
}
