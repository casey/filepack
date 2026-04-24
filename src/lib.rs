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
//! exists only to facilitate code-sharing between the `filepack` binary and
//! integration tests. As such, it should not be used by outside consumers, and
//! provides no semantic versioning guarantees.

use {
  self::{
    archive::Archive,
    archive_builder::ArchiveBuilder,
    archive_error::ArchiveError,
    arguments::Arguments,
    bech32_decoder::Bech32Decoder,
    bech32_encoder::Bech32Encoder,
    bech32_error::Bech32Error,
    bech32_type::Bech32Type,
    component::Component,
    component_buf::ComponentBuf,
    component_error::ComponentError,
    count::Count,
    dalek_signature_error::DalekSignatureError,
    date_time::DateTime,
    decode::Decode,
    decode_error::DecodeError,
    decoder::Decoder,
    directory::Directory,
    directory_tree_entry::DirectoryTreeEntry,
    display_path::DisplayPath,
    display_secret::DisplaySecret,
    entries::Entries,
    entry::Entry,
    entry_type::EntryType,
    file::File,
    format::Format,
    functions::{current_dir, decode_path, default, is_lowercase_hex, now},
    hash_error::HashError,
    head::Head,
    key_identifier::KeyIdentifier,
    key_name::KeyName,
    key_type::KeyType,
    keychain::Keychain,
    language::Language,
    lint_error::{Lint, LintError},
    lint_group::LintGroup,
    map_decoder::MapDecoder,
    metadata::Metadata,
    mode::Mode,
    options::Options,
    owo_colorize_ext::OwoColorizeExt,
    package::Package,
    path_error::PathError,
    private_key_error::PrivateKeyError,
    public_key_error::PublicKeyError,
    sign_options::SignOptions,
    signature_error::SignatureError,
    style::Style,
    subcommand::Subcommand,
    tag::Tag,
    ticked::Ticked,
    url::Url,
    utf8_path_ext::Utf8PathExt,
    version::Version,
  },
  bech32::{
    ByteIterExt, Fe32, Fe32IterExt, Hrp,
    primitives::decode::{CheckedHrpstring, CheckedHrpstringError},
  },
  blake3::Hasher,
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::{Parser, ValueEnum},
  filepack_cbor::{Decode, Encode},
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
  num_traits::One,
  owo_colors::Styled,
  regex::Regex,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  serde_with::{
    DeserializeFromStr, MapPreventDuplicates, SerializeDisplay, SetPreventDuplicates, serde_as,
    skip_serializing_none,
  },
  snafu::{ErrorCompat, OptionExt, ResultExt, Snafu, ensure},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    borrow::Borrow,
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap},
    env,
    fmt::{self, Debug, Display, Formatter, Write},
    fs::{self, Permissions},
    io::{self, IsTerminal},
    iter,
    marker::PhantomData,
    num::TryFromIntError,
    ops::Deref,
    path::{Path, PathBuf},
    process, ptr,
    str::{self, FromStr, Utf8Error},
    sync::LazyLock,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
  },
  strum::{EnumDiscriminants, EnumIter, EnumString, FromRepr, IntoEnumIterator, IntoStaticStr},
  usized::IntoU64,
  walkdir::WalkDir,
};

pub use self::{
  directory_tree::DirectoryTree, encode::Encode, encoder::Encoder, error::Error,
  fingerprint::Fingerprint, hash::Hash, language_error::LanguageError, major_type::MajorType,
  manifest::Manifest, map_encoder::MapEncoder, message::Message, private_key::PrivateKey,
  public_key::PublicKey, relative_path::RelativePath, signature::Signature, tag_error::TagError,
};

#[cfg(test)]
use strum::IntoDiscriminant;

#[cfg(test)]
fn tempdir() -> tempfile::TempDir {
  tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap()
}

#[cfg(test)]
#[track_caller]
fn assert_cbor<T: Debug + Decode + Encode + PartialEq>(value: T, cbor: &[u8]) {
  let buffer = value.encode_to_vec();
  assert_eq!(buffer, cbor);
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[cfg(test)]
#[track_caller]
fn assert_encoding<T: Debug + Decode + Encode + PartialEq>(value: T) {
  let buffer = value.encode_to_vec();
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[macro_export]
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
macro_rules! assert_matches_regex {
  ($haystack:expr, $re:expr $(,)?) => {{
    let haystack = $haystack;
    let re = Regex::new(&$re).unwrap();
    assert!(
      re.is_match(&haystack),
      "assertion failed: `{haystack:?}` does not match `{}`",
      re.as_str(),
    );
  }};
}

macro_rules! count_some {
  ($($option:expr),* $(,)?) => {
    0u64 $(+ u64::from($option.is_some()))*
  };
}

mod archive;
mod archive_builder;
mod archive_error;
mod arguments;
mod bech32_decoder;
mod bech32_encoder;
mod bech32_error;
mod bech32_type;
mod component;
mod component_buf;
mod component_error;
mod count;
mod dalek_signature_error;
mod date_time;
mod decode;
mod decode_error;
mod decoder;
mod directory;
mod directory_tree;
mod directory_tree_entry;
mod display_path;
mod display_secret;
mod encode;
mod encoder;
mod entries;
mod entry;
mod entry_type;
mod error;
mod file;
mod filename;
mod filesystem;
mod fingerprint;
mod format;
mod functions;
mod hash;
mod hash_error;
mod head;
mod key_identifier;
mod key_name;
mod key_type;
mod keychain;
mod language;
mod language_error;
mod lint_error;
mod lint_group;
mod major_type;
mod manifest;
mod map_decoder;
mod map_encoder;
mod message;
mod metadata;
mod mode;
mod options;
mod owo_colorize_ext;
mod package;
mod path_error;
mod private_key;
mod private_key_error;
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
mod tag;
mod tag_error;
mod ticked;
mod url;
mod utf8_path_ext;
mod version;

#[cfg(test)]
mod derive;
#[cfg(test)]
mod test;

const BECH32_VERSION: Fe32 = Fe32::A;

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
