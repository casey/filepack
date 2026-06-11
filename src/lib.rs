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
    application::Application,
    archive::Archive,
    archive_builder::ArchiveBuilder,
    archive_error::ArchiveError,
    arguments::Arguments,
    array_decoder::ArrayDecoder,
    audio_type::AudioType,
    authenticated::Authenticated,
    bech32_decoder::Bech32Decoder,
    bech32_encoder::Bech32Encoder,
    bech32_error::Bech32Error,
    bech32_type::Bech32Type,
    cbor::Cbor,
    checked_url::CheckedUrl,
    component::Component,
    component_error::ComponentError,
    context::Context,
    count::Count,
    dalek_signature_error::DalekSignatureError,
    database_metadata::DatabaseMetadata,
    date_time::DateTime,
    decode_error::DecodeError,
    dimensions::Dimensions,
    directory_tree_entry::DirectoryTreeEntry,
    display_path::DisplayPath,
    display_secret::DisplaySecret,
    entries::Entries,
    envelope::Envelope,
    file::File,
    filename::Filename,
    format::Format,
    functions::{
      client, current_dir, decode_path, default, is_lowercase_hex, now, parse_server_url,
      transfer_tempfile,
    },
    hash_error::HashError,
    hashing_writer::HashingWriter,
    head::Head,
    image_type::ImageType,
    key_identifier::KeyIdentifier,
    key_name::KeyName,
    key_type::KeyType,
    keychain::Keychain,
    language::Language,
    lint_error::{Lint, LintError},
    lint_group::LintGroup,
    map_decoder::MapDecoder,
    media::{Media, MediaType},
    mode::Mode,
    options::Options,
    or::Or,
    ordinal::Ordinal,
    owo_colorize_ext::OwoColorizeExt,
    package::Package,
    path_error::PathError,
    private_key_error::PrivateKeyError,
    public_key_error::PublicKeyError,
    reqwest_response_ext::ReqwestResponseExt,
    reqwest_result_ext::ReqwestResultExt,
    resource::Resource,
    resource_type::ResourceType,
    server::Server,
    server_error::ServerError,
    sign_options::SignOptions,
    signature_error::SignatureError,
    static_asset::StaticAsset,
    style::Style,
    subcommand::Subcommand,
    tag::Tag,
    ticked::Ticked,
    token::Token,
    track::Track,
    type_name::TypeName,
    utf8_path_ext::Utf8PathExt,
  },
  axum::{
    body::Body,
    http::{self, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
  },
  axum_extra::{TypedHeader, headers},
  bech32::{
    ByteIterExt, Fe32, Fe32IterExt, Hrp,
    primitives::decode::{CheckedHrpstring, CheckedHrpstringError},
  },
  blake3::Hasher,
  boilerplate::{Boilerplate, Trusted},
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::{ArgGroup, Parser, ValueEnum},
  filepack_cbor::{Decode, DecodeFromStr, Encode, EncodeDisplay},
  futures_util::StreamExt,
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
  md5::{Digest, Md5},
  mime::Mime,
  num_traits::One,
  owo_colors::Styled,
  regex::Regex,
  reqwest::blocking::Client,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  serde_with::{
    DeserializeFromStr, MapPreventDuplicates, SerializeDisplay, SetPreventDuplicates, serde_as,
    skip_serializing_none,
  },
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu, ensure},
  std::{
    array,
    backtrace::{Backtrace, BacktraceStatus},
    borrow::Borrow,
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env,
    fmt::{self, Debug, Display, Formatter},
    fs::{self, Permissions},
    io::{self, IsTerminal, Seek, SeekFrom, Write},
    iter,
    marker::PhantomData,
    net::SocketAddr,
    num::{NonZeroUsize, ParseIntError, TryFromIntError},
    ops::{Bound, Deref},
    path::{Path, PathBuf},
    process, ptr,
    str::{self, FromStr, Utf8Error},
    sync::{
      Arc, LazyLock,
      atomic::{self, AtomicU64},
    },
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
  },
  strum::{
    Display, EnumDiscriminants, EnumIter, EnumString, FromRepr, IntoDiscriminant, IntoEnumIterator,
    IntoStaticStr,
  },
  tempfile::NamedTempFile,
  tokio::io::{AsyncReadExt, AsyncWriteExt},
  tokio_util::io::ReaderStream,
  url::Url,
  usized::IntoU64,
  walkdir::WalkDir,
  zune_jpeg::JpegDecoder,
};

pub use self::{
  array_encoder::ArrayEncoder, component_buf::ComponentBuf, decode::Decode, decoder::Decoder,
  directory::Directory, directory_tree::DirectoryTree, encode::Encode, encoder::Encoder,
  entry::Entry, entry_type::EntryType, error::Error, fingerprint::Fingerprint,
  functions::install_default_crypto_provider, hash::Hash, language_error::LanguageError,
  major_type::MajorType, manifest::Manifest, map_encoder::MapEncoder, metadata::Metadata,
  private_key::PrivateKey, public_key::PublicKey, relative_path::RelativePath,
  signature::Signature, sorted_set::SortedSet, statement::Statement, tag_error::TagError,
  version::Version,
};

#[cfg(test)]
use {
  std::assert_matches,
  tempfile::TempDir,
  test::{EMPTY_MD5, assert_cbor, assert_cbor_eq, assert_encoding, flac, tempdir},
  unindent::unindent,
};

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

pub mod api;
mod application;
mod archive;
mod archive_builder;
mod archive_error;
mod arguments;
mod array_decoder;
mod array_encoder;
mod audio_type;
mod authenticated;
mod bech32_decoder;
mod bech32_encoder;
mod bech32_error;
mod bech32_type;
mod cbor;
mod checked_url;
mod component;
mod component_buf;
mod component_error;
mod context;
mod count;
mod dalek_signature_error;
mod database_metadata;
mod date_time;
mod decode;
mod decode_error;
mod decoder;
mod dimensions;
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
mod envelope;
mod error;
mod file;
mod filename;
mod filesystem;
mod fingerprint;
mod format;
mod functions;
mod hash;
mod hash_error;
mod hashing_writer;
mod head;
mod image_type;
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
mod media;
mod metadata;
mod mode;
mod options;
mod or;
mod ordinal;
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
mod reqwest_response_ext;
mod reqwest_result_ext;
mod resource;
mod resource_type;
mod server;
mod server_error;
mod sign_options;
mod signature;
mod signature_error;
mod sorted_set;
mod statement;
mod static_asset;
mod style;
mod subcommand;
mod tag;
mod tag_error;
pub mod templates;
mod ticked;
mod token;
mod track;
mod type_name;
mod utf8_path_ext;
mod version;

#[cfg(test)]
mod derive;
#[cfg(test)]
mod test;

const BECH32_VERSION: Fe32 = Fe32::A;
const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

type ServerResult<T = (), E = ServerError> = std::result::Result<T, E>;
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
