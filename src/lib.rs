//! `filepack` is a tool for hashing directories.
//!
//! `filepack create` creates a manifest which commits to the content of the
//! current directory and its children.
//!
//! `filepack verify` verifies a manifest against the content of the current
//! directory of its children.
//!
//! This can be used to detect accidental corruption or modification. If
//! `filepack verify` succeeds, the contents of the directory have not changed.
//!
//! A manifest can also be used to guard against intentional corruption, as
//! long as the manifest itself is kept secure.
//!
//! The `filepack` library crate is not intended for general consumption, and
//! exists mainly to facilitate code-sharing between the `filepack` binary and
//! integration tests. As such, it provides no semantic versioning guarantees.

use {
  self::{
    arguments::Arguments,
    component::Component,
    count::Count,
    digest::Digest,
    display_path::DisplayPath,
    display_secret::DisplaySecret,
    entries::Entries,
    fingerprint_hasher::FingerprintHasher,
    fingerprint_prefix::FingerprintPrefix,
    functions::{current_dir, decode_path, default, is_default, is_lowercase_hex, now},
    index::Index,
    key_identifier::KeyIdentifier,
    key_name::KeyName,
    key_type::KeyType,
    keychain::Keychain,
    lint::Lint,
    lint_group::LintGroup,
    message::Message,
    metadata::Metadata,
    mode::Mode,
    options::Options,
    owo_colorize_ext::OwoColorizeExt,
    path_error::PathError,
    public_key_error::PublicKeyError,
    sign_options::SignOptions,
    signature_error::SignatureError,
    style::Style,
    subcommand::Subcommand,
    template::Template,
    utf8_path_ext::Utf8PathExt,
  },
  blake3::Hasher,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
  num_traits::One,
  owo_colors::Styled,
  regex::Regex,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  serde_with::{DeserializeFromStr, MapPreventDuplicates, SerializeDisplay, serde_as},
  snafu::{ErrorCompat, OptionExt, ResultExt, Snafu, ensure},
  std::{
    array::TryFromSliceError,
    backtrace::{Backtrace, BacktraceStatus},
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    env,
    fmt::{self, Display, Formatter},
    fs::{self, Permissions},
    io::{self, IsTerminal},
    iter,
    path::{Path, PathBuf},
    process,
    str::{self, FromStr},
    sync::LazyLock,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
  },
  strum::{EnumIter, EnumString, IntoStaticStr},
  usized::IntoU64,
  walkdir::WalkDir,
};

pub use self::{
  directory::Directory, entry::Entry, error::Error, file::File, fingerprint::Fingerprint,
  hash::Hash, manifest::Manifest, note::Note, private_key::PrivateKey, public_key::PublicKey,
  relative_path::RelativePath, signature::Signature,
};

#[cfg(test)]
use {std::collections::HashSet, strum::IntoEnumIterator};

#[cfg(test)]
fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap()
}

mod arguments;
mod component;
mod count;
mod digest;
mod directory;
mod display_path;
mod display_secret;
mod entries;
mod entry;
mod error;
mod file;
mod filesystem;
mod fingerprint;
mod fingerprint_hasher;
mod fingerprint_prefix;
mod functions;
mod hash;
mod index;
mod key_identifier;
mod key_name;
mod key_type;
mod keychain;
mod lint;
mod lint_group;
mod manifest;
mod message;
mod metadata;
mod mode;
mod note;
mod options;
mod owo_colorize_ext;
mod path_error;
mod private_key;
mod progress_bar;
mod public_key;
mod public_key_error;
mod re;
mod relative_path;
mod sign_options;
mod signature;
mod signature_error;
mod style;
mod subcommand;
mod template;
mod utf8_path_ext;

#[cfg(test)]
mod test;

const SEPARATORS: [char; 2] = ['/', '\\'];

type Result<T = (), E = Error> = std::result::Result<T, E>;

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
