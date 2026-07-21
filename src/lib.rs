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
    audio::Audio,
    audio_error::AudioError,
    audio_format::AudioFormat,
    audio_info::AudioInfo,
    audio_type::AudioType,
    authenticated::Authenticated,
    bech32_decoder::Bech32Decoder,
    bech32_encoder::Bech32Encoder,
    bech32_error::Bech32Error,
    bech32_type::Bech32Type,
    cause::Cause,
    cbor::Cbor,
    checked_url::CheckedUrl,
    codec::Codec,
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
    display_duration::DisplayDuration,
    display_path::DisplayPath,
    display_secret::DisplaySecret,
    entries::Entries,
    envelope::Envelope,
    file::File,
    format::Format,
    functions::{
      client, current_dir, decode_path, default, format_size, is_lowercase_hex, now,
      parse_server_url, transfer_tempfile,
    },
    hash_error::HashError,
    hashing_writer::HashingWriter,
    head::Head,
    image::Image,
    image_error::ImageError,
    image_format::ImageFormat,
    image_type::ImageType,
    iso8601_duration::Iso8601Duration,
    item::Item,
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
    percent_encode::PercentEncode,
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
    templates::PageHtml,
    text_error::TextError,
    ticked::Ticked,
    token::Token,
    totals_error::TotalsError,
    track::Track,
    track_info::TrackInfo,
    type_name::TypeName,
    utf8_path_ext::Utf8PathExt,
    validate::Validate,
    version::Version,
    video::Video,
    video_error::VideoError,
    video_format::VideoFormat,
    video_type::VideoType,
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
  claxon::FlacReader,
  filepack_cbor::{Decode, DecodeFromStr, Encode, EncodeDisplay},
  futures_util::StreamExt,
  humansize::{BINARY, FormatSizeOptions, SizeFormatter},
  indicatif::{ProgressBar, ProgressStyle},
  lexiclean::Lexiclean,
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
    io::{self, BufReader, IsTerminal, Read, Seek, SeekFrom, Write},
    iter,
    net::SocketAddr,
    num::{NonZeroUsize, ParseIntError, TryFromIntError},
    ops::{Bound, Deref},
    path::{Path, PathBuf},
    process::{self, ExitCode},
    ptr,
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
  array_encoder::ArrayEncoder,
  component_buf::ComponentBuf,
  decode::Decode,
  decoder::Decoder,
  directory::Directory,
  directory_ext::DirectoryExt,
  directory_tree::DirectoryTree,
  encode::Encode,
  encoder::Encoder,
  entry::{Entry, EntryType},
  error::Error,
  fingerprint::Fingerprint,
  functions::install_default_crypto_provider,
  hash::Hash,
  language_error::LanguageError,
  major_type::MajorType,
  manifest::Manifest,
  map_encoder::MapEncoder,
  metadata::Metadata,
  mp4_builder::Mp4Builder,
  page::Page,
  private_key::PrivateKey,
  public_key::PublicKey,
  relative_path::RelativePath,
  signature::Signature,
  sorted_set::SortedSet,
  statement::Statement,
  text::Text,
  totals::Totals,
};

#[cfg(test)]
use {
  std::assert_matches,
  tempfile::TempDir,
  test::{assert_cbor, assert_cbor_eq, assert_encoding, flac, tempdir},
  unindent::unindent,
  webm_builder::WebmBuilder,
};

#[cfg(test)]
macro_rules! assert_matches_regex {
  ($haystack:expr, $pattern:expr $(,)?) => {{
    let haystack = $haystack;
    let re = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    if !re.is_match(haystack.as_ref()) {
      eprintln!("Regex did not match:");
      pretty_assertions::assert_eq!(re.as_str(), haystack);
    }
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
mod audio;
mod audio_error;
mod audio_format;
mod audio_info;
mod audio_type;
mod authenticated;
mod bech32_decoder;
mod bech32_encoder;
mod bech32_error;
mod bech32_type;
mod cause;
mod cbor;
mod checked_url;
mod codec;
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
mod directory_ext;
mod directory_tree;
mod directory_tree_entry;
mod display_duration;
mod display_path;
mod display_secret;
mod encode;
mod encoder;
mod entries;
mod entry;
mod envelope;
mod error;
mod file;
mod filesystem;
mod fingerprint;
mod format;
mod functions;
mod hash;
mod hash_error;
mod hashing_writer;
mod head;
mod image;
mod image_error;
mod image_format;
mod image_type;
mod iso8601_duration;
mod item;
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
mod mp4_builder;
mod options;
mod or;
mod ordinal;
mod owo_colorize_ext;
mod package;
mod page;
mod path_error;
mod percent_encode;
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
pub mod templates;
mod text;
mod text_error;
mod ticked;
mod token;
mod totals;
mod totals_error;
mod track;
mod track_info;
mod type_name;
mod utf8_path_ext;
mod validate;
mod version;
mod video;
mod video_error;
mod video_format;
mod video_type;

#[cfg(test)]
mod derive;
#[cfg(test)]
mod test;
#[cfg(test)]
mod webm_builder;

const BECH32_VERSION: Fe32 = Fe32::A;
const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

type PageResult<T> = ServerResult<PageHtml<T>>;
type Result<T = (), E = Error> = std::result::Result<T, E>;
type ServerResult<T = (), E = ServerError> = std::result::Result<T, E>;

fn initialize_tracing() -> Result<(), Box<dyn std::error::Error>> {
  use {
    tracing::level_filters::LevelFilter,
    tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt},
  };

  #[cfg(not(target_os = "linux"))]
  use tracing_subscriber::layer::Identity;

  #[cfg(target_os = "linux")]
  fn journal_layer() -> Result<Option<tracing_journald::Layer>, Box<dyn std::error::Error>> {
    if std::env::var_os("JOURNAL_STREAM").is_some_and(|value| !value.is_empty()) {
      Ok(Some(tracing_journald::layer()?))
    } else {
      Ok(None)
    }
  }

  #[cfg(not(target_os = "linux"))]
  #[allow(clippy::unnecessary_wraps)]
  fn journal_layer() -> Result<Option<Identity>, Box<dyn std::error::Error>> {
    Ok(None)
  }

  let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::ERROR.into())
    .with_env_var("FILEPACK_LOG")
    .from_env()?;

  let journal = journal_layer()?;

  let stderr = journal
    .is_none()
    .then(|| tracing_subscriber::fmt::layer().with_writer(io::stderr));

  tracing_subscriber::registry()
    .with(filter)
    .with(journal)
    .with(stderr)
    .try_init()?;

  Ok(())
}

pub fn run() -> ExitCode {
  if let Err(err) = initialize_tracing() {
    eprintln!("failed to initialize tracing: {err}");
    return ExitCode::FAILURE;
  }

  if let Err(err) = Arguments::parse().run() {
    let style = Style::stderr();
    eprintln!(
      "{}: {}",
      "error".style(style.error()),
      err.style(style.message()),
    );

    let causes = err.causes();
    for (i, cause) in causes.iter().enumerate() {
      eprintln!(
        "       {}─ {cause}",
        if i == causes.len() - 1 { '└' } else { '├' }
      );
    }

    if let Some(backtrace) = err.backtrace()
      && backtrace.status() == BacktraceStatus::Captured
    {
      eprintln!();
      eprintln!("backtrace:");
      eprintln!("{backtrace}");
    }

    return ExitCode::FAILURE;
  }

  ExitCode::SUCCESS
}
