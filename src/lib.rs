//! `filepack` is a tool for hashing directories.
//!
//! `filepack create` creates a manifest which commits to the content of the
//! current directory and its children.
//!
//! `filepack verify` verifies a manifest against the content of the current
//! directory and its children.
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

// Building without the default `client`, `fs` and `server` features leaves
// code that only they use unreachable.
#![cfg_attr(
  not(all(feature = "client", feature = "fs", feature = "server")),
  allow(dead_code, unreachable_pub, unused_imports)
)]

use {
  self::{
    application::Application,
    archive::Archive,
    archive_builder::ArchiveBuilder,
    archive_error::ArchiveError,
    array_decoder::ArrayDecoder,
    array_encoder::ArrayEncoder,
    audio::Audio,
    audio_codec::AudioCodec,
    audio_error::AudioError,
    audio_format::AudioFormat,
    audio_metadata::AudioMetadata,
    audio_position_error::AudioPositionError,
    authorization_error::AuthorizationError,
    bit_reader::BitReader,
    bytes_error::BytesError,
    cause::Cause,
    checked_url::CheckedUrl,
    chroma_subsampling::ChromaSubsampling,
    claims::Claims,
    color_info::ColorInfo,
    color_type::ColorType,
    component::Component,
    component_buf::ComponentBuf,
    component_error::ComponentError,
    compression::Compression,
    content::Content,
    context::Context,
    date::Date,
    deco_request::DecoRequest,
    deco_response::DecoResponse,
    decode_error::DecodeError,
    decode_owned::DecodeOwned,
    descendants::Descendants,
    describe::Describe,
    dimensions::Dimensions,
    directory_tree::DirectoryTree,
    directory_tree_entry::DirectoryTreeEntry,
    display_bitrate::DisplayBitrate,
    display_bits_per_pixel::DisplayBitsPerPixel,
    display_duration::DisplayDuration,
    display_frame_rate::DisplayFrameRate,
    display_millis::DisplayMillis,
    display_path::DisplayPath,
    display_private_key::DisplayPrivateKey,
    display_sample_rate::DisplaySampleRate,
    domain::Domain,
    ed25519_signature::Ed25519Signature,
    embedded_image::EmbeddedImage,
    entry_info::{EntryInfo, EntryType},
    envelope::Envelope,
    error::Error,
    exif_decoder::ExifDecoder,
    exif_error::ExifError,
    file::File,
    filesystem_error::FilesystemError,
    flac_decoder::FlacDecoder,
    float_ext::FloatExt,
    format::Format,
    functions::{check_case, current_dir, decode_path, default, format_size, parse_number},
    head::Head,
    hex::Hex,
    hex_error::HexError,
    host::Host,
    host_error::HostError,
    image::Image,
    image_format::ImageFormat,
    image_metadata::ImageMetadata,
    info::Info,
    info_builder::InfoBuilder,
    invalid_public_key::InvalidPublicKey,
    iso8601_duration::Iso8601Duration,
    item::Item,
    items::Items,
    json::Json,
    key_identifier::KeyIdentifier,
    key_identifier_error::KeyIdentifierError,
    key_name::KeyName,
    key_type::KeyType,
    language_error::LanguageError,
    lint_error::LintError,
    lint_selector::LintSelector,
    m4a_decoder::M4aDecoder,
    m4a_error::M4aError,
    magic::Magic,
    malformed_error::MalformedError,
    map_decoder::MapDecoder,
    map_encoder::MapEncoder,
    media::Media,
    media_item::MediaItem,
    media_item_resource::MediaItemResource,
    message::Message,
    mode::Mode,
    mount::Mount,
    mount_error::MountError,
    mp3_decoder::Mp3Decoder,
    mp3_error::Mp3Error,
    mp4_decoder::Mp4Decoder,
    mp4_error::Mp4Error,
    number_error::NumberError,
    open_graph_image::OpenGraphImage,
    or::Or,
    order::Order,
    ordinal::Ordinal,
    orientation::Orientation,
    owo_colorize_ext::OwoColorizeExt,
    package_identifier_error::PackageIdentifierError,
    package_metadata::PackageMetadata,
    package_summary::PackageSummary,
    page_error::PageError,
    path_error::PathError,
    path_extractor::PathExtractor,
    percent_encode::PercentEncode,
    private_key_error::PrivateKeyError,
    public_key_error::PublicKeyError,
    relative_path::RelativePath,
    resolved::Resolved,
    resource_type::ResourceType,
    revision::Revision,
    rotation::Rotation,
    server_error::ServerError,
    server_url::ServerUrl,
    server_url_error::ServerUrlError,
    shortcut::Shortcut,
    sign_options::SignOptions,
    signature::Signature,
    signature_entry::SignatureEntry,
    signature_error::SignatureError,
    sort::Sort,
    sort_key::SortKey,
    sorted_set::SortedSet,
    statement::Statement,
    style::Style,
    templates::{ErrorHtml, PageHtml},
    text::Text,
    text_error::TextError,
    ticked::Ticked,
    time::Time,
    time_error::TimeError,
    totals_error::TotalsError,
    track::Track,
    track_codec::TrackCodec,
    track_info::TrackInfo,
    unknown_error::UnknownError,
    url_error::UrlError,
    utf8_path_ext::Utf8PathExt,
    validate::Validate,
    video::Video,
    video_error::VideoError,
    video_format::VideoFormat,
    video_metadata::VideoMetadata,
    webm_decoder::WebmDecoder,
    xmp_error::XmpError,
  },
  axum::{
    body::Body,
    http::{self, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
  },
  axum_extra::{TypedHeader, headers},
  blake3::Hasher,
  boilerplate::{Boilerplate, Trusted},
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  claxon::FlacReader,
  filepack_derive::{Decode, DecodeFromStr, Encode, EncodeDisplay, Json, Magic, route},
  futures_util::StreamExt,
  humansize::{BINARY, BaseUnit, DECIMAL, FormatSizeOptions, SizeFormatter},
  icu_normalizer::ComposingNormalizerBorrowed,
  id3::TagLike,
  lexiclean::Lexiclean,
  mime::Mime,
  num_traits::One,
  owo_colors::Styled,
  regex::Regex,
  serde::{Deserialize, Serialize, Serializer},
  serde_with::{
    DeserializeFromStr, MapPreventDuplicates, SerializeDisplay, SetPreventDuplicates, serde_as,
    skip_serializing_none,
  },
  snafu::{ErrorCompat, IntoError, OptionExt, ResultExt, Snafu, ensure},
  std::{
    backtrace::{Backtrace, BacktraceStatus},
    borrow::Borrow,
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    convert::Infallible,
    env,
    fmt::{self, Debug, Display, Formatter},
    fs::{self, Permissions},
    io::{self, BufReader, IsTerminal, Read, Seek, SeekFrom, Write},
    iter, mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
    num::{NonZeroUsize, ParseIntError},
    ops::{Bound, Deref},
    path::{Path, PathBuf},
    process::{self, ExitCode},
    ptr, slice,
    str::{self, FromStr, Utf8Error},
    sync::{
      Arc, LazyLock,
      atomic::{self, AtomicU64},
    },
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
    vec,
  },
  strum::{
    Display, EnumDiscriminants, EnumIter, EnumString, FromRepr, IntoDiscriminant, IntoEnumIterator,
    IntoStaticStr, VariantArray,
  },
  tokio::io::{AsyncReadExt, AsyncWriteExt},
  tokio_util::io::ReaderStream,
  unicase::UniCase,
  unindent::unindent,
  url::Url,
  usized::IntoU64,
  zune_jpeg::{JpegDecoder, SampleRatios, zune_core::colorspace::ColorSpace},
};

#[cfg(feature = "fs")]
use {
  clap::{ArgGroup, CommandFactory, Parser, ValueEnum, ValueHint},
  clap_complete::{ArgValueCandidates, CompleteEnv, CompletionCandidate},
  tempfile::{NamedTempFile, TempPath},
  walkdir::WalkDir,
};

#[cfg(feature = "fs")]
use self::{
  arguments::Arguments, hashing_writer::HashingWriter, keychain::Keychain, linter::Linter,
  options::Options, output_format::OutputFormat, output_format_json::OutputFormatJson,
  progress_bar::ProgressBar, shell::Shell, subcommand::Subcommand, walk::Walk,
};

#[cfg(feature = "fs")]
use self::functions::{check_metadata, transfer_tempfile, verify_subpackage};

#[cfg(feature = "fs")]
pub use self::{loader::Loader, state::State};

#[cfg(feature = "redb")]
use self::{database_metadata::DatabaseMetadata, type_name::TypeName};

#[cfg(feature = "server")]
use self::{
  authenticated::Authenticated, resource::Resource, server::Server, static_asset::StaticAsset,
  subcommand::serve::route::Route,
};

#[cfg(feature = "client")]
use self::{client::Client, reqwest_response_ext::ReqwestResponseExt};

#[cfg(any(feature = "client", feature = "server"))]
pub use self::functions::install_default_crypto_provider;

#[cfg(test)]
use {
  jpeg_builder::JpegBuilder,
  std::assert_matches,
  tempfile::TempDir,
  test::{
    assert_argument_conflict, assert_deco, assert_deco_eq, assert_encoding,
    assert_invalid_argument_value, assert_missing_argument, assert_redb_impls, exif, tempdir,
    with_unknown_field,
  },
  webm_builder::WebmBuilder,
};

pub use self::{
  bytes::Bytes,
  content_hasher::ContentHasher,
  count::Count,
  decode::Decode,
  decode_options::DecodeOptions,
  decoded::Decoded,
  decoder::Decoder,
  directory::Directory,
  directory_ext::DirectoryExt,
  encode::Encode,
  encoder::Encoder,
  entry::Entry,
  fingerprint::Fingerprint,
  flac_builder::FlacBuilder,
  functions::{gradient, gradient_alpha, now},
  hash::Hash,
  key_error::KeyError,
  language::Language,
  lint_error::Lint,
  lint_group::LintGroup,
  magic_type::MagicType,
  manifest::Manifest,
  manifest_error::ManifestError,
  media::MediaType,
  metadata::Metadata,
  metadata_error::MetadataError,
  mp3_builder::Mp3Builder,
  mp4_builder::Mp4Builder,
  non_empty_ordered_set::NonEmptyOrderedSet,
  package_identifier::PackageIdentifier,
  page::Page,
  png_builder::PngBuilder,
  private_key::PrivateKey,
  public_key::PublicKey,
  reference::Reference,
  revision_object::RevisionObject,
  server_state::ServerState,
  statement_error::StatementError,
  tag::Tag,
  totals::Totals,
  unstable_feature::UnstableFeature,
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

mod api;
mod application;
mod archive;
mod archive_builder;
mod archive_error;
#[cfg(feature = "fs")]
mod arguments;
mod array_decoder;
mod array_encoder;
mod audio;
mod audio_codec;
mod audio_error;
mod audio_format;
mod audio_metadata;
mod audio_position_error;
#[cfg(feature = "server")]
mod authenticated;
mod authorization_error;
mod bit_reader;
mod bytes;
mod bytes_error;
mod cause;
mod checked_url;
mod chroma_subsampling;
mod claims;
#[cfg(feature = "client")]
mod client;
mod color_info;
mod color_type;
mod component;
mod component_buf;
mod component_error;
mod compression;
mod content;
mod content_hasher;
mod context;
mod count;
#[cfg(feature = "redb")]
mod database_metadata;
mod date;
pub mod deco;
mod deco_request;
mod deco_response;
mod decode;
mod decode_error;
mod decode_options;
mod decode_owned;
mod decoded;
mod decoder;
mod descendants;
mod describe;
mod dimensions;
mod directory;
mod directory_ext;
mod directory_tree;
mod directory_tree_entry;
mod display_bitrate;
mod display_bits_per_pixel;
mod display_duration;
mod display_frame_rate;
mod display_millis;
mod display_path;
mod display_private_key;
mod display_sample_rate;
mod domain;
mod ed25519_signature;
mod embedded_image;
mod encode;
mod encoder;
mod entry;
mod entry_info;
mod envelope;
mod error;
mod exif_decoder;
mod exif_error;
mod file;
// Lets the derive macros in `filepack-derive` name this crate as `::filepack`.
extern crate self as filepack;
mod filesystem;
mod filesystem_error;
mod fingerprint;
mod flac_builder;
mod flac_decoder;
mod float_ext;
mod format;
mod functions;
mod hash;
#[cfg(feature = "fs")]
mod hashing_writer;
mod head;
pub mod hex;
mod hex_error;
mod host;
mod host_error;
mod image;
mod image_format;
mod image_metadata;
mod info;
mod info_builder;
mod invalid_public_key;
mod iso8601_duration;
mod item;
mod items;
pub mod json;
mod key_error;
mod key_identifier;
mod key_identifier_error;
mod key_name;
mod key_type;
#[cfg(feature = "fs")]
mod keychain;
mod language;
mod language_error;
mod lint_error;
mod lint_group;
mod lint_selector;
#[cfg(feature = "fs")]
mod linter;
#[cfg(feature = "fs")]
mod loader;
mod m4a_decoder;
mod m4a_error;
mod magic;
mod magic_type;
mod malformed_error;
mod manifest;
mod manifest_error;
mod map_decoder;
mod map_encoder;
mod media;
mod media_item;
mod media_item_resource;
mod message;
mod metadata;
mod metadata_error;
mod mode;
mod mount;
mod mount_error;
mod mp3_builder;
mod mp3_decoder;
mod mp3_error;
mod mp4;
mod mp4_builder;
mod mp4_decoder;
mod mp4_error;
mod non_empty_ordered_set;
mod number_error;
mod open_graph_image;
#[cfg(feature = "fs")]
mod options;
mod or;
mod order;
mod ordinal;
mod orientation;
#[cfg(feature = "fs")]
mod output_format;
#[cfg(feature = "fs")]
mod output_format_json;
mod owo_colorize_ext;
mod package_identifier;
mod package_identifier_error;
mod package_metadata;
mod package_summary;
mod page;
mod page_error;
mod path_error;
mod path_extractor;
mod percent_encode;
mod png_builder;
mod private_key;
mod private_key_error;
#[cfg(feature = "fs")]
mod progress_bar;
mod public_key;
mod public_key_error;
mod re;
pub mod reference;
mod relative_path;
#[cfg(feature = "client")]
mod reqwest_response_ext;
mod resolved;
#[cfg(feature = "server")]
mod resource;
mod resource_type;
mod revision;
mod revision_object;
mod rotation;
#[cfg(feature = "server")]
mod server;
mod server_error;
mod server_state;
mod server_url;
mod server_url_error;
#[cfg(feature = "fs")]
mod shell;
mod shortcut;
mod sign_options;
mod signature;
mod signature_entry;
mod signature_error;
mod sort;
mod sort_key;
mod sorted_set;
#[cfg(feature = "fs")]
mod state;
mod statement;
mod statement_error;
#[cfg(feature = "server")]
mod static_asset;
mod style;
#[cfg(feature = "fs")]
mod subcommand;
mod tag;
pub mod templates;
mod text;
mod text_error;
mod ticked;
mod time;
mod time_error;
mod totals;
mod totals_error;
mod track;
mod track_codec;
mod track_info;
#[cfg(feature = "redb")]
mod type_name;
mod unknown_error;
mod unstable_feature;
mod url_error;
mod utf8_path_ext;
mod validate;
mod video;
mod video_error;
mod video_format;
mod video_metadata;
#[cfg(feature = "fs")]
mod walk;
mod webm_decoder;
mod xmp;
mod xmp_error;
pub mod yaml;

#[cfg(test)]
mod derive;
#[cfg(test)]
mod derive_hygiene;
#[cfg(test)]
mod jpeg_builder;
#[cfg(test)]
mod test;
#[cfg(test)]
mod webm_builder;

#[doc(hidden)]
pub(crate) mod __private {
  pub(crate) use {
    crate::{
      Decode, DecodeResult, Decoder, Encode, Encoder, Magic, MagicType, MalformedError,
      UnknownError, Validate, deco, json, malformed_error,
    },
    snafu::ensure,
  };

  #[cfg(feature = "server")]
  pub(crate) use crate::{Describe, reference, subcommand::serve::route::Route};
}

const API_VERSION: u64 = 0;
const API_VERSION_HEADER: HeaderName = HeaderName::from_static("filepack-api-version");

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

type Result<T = (), E = Error> = std::result::Result<T, E>;

type DecodeResult<T = ()> = Result<T, DecodeError>;
type FilesystemResult<T = ()> = Result<T, FilesystemError>;
type PageResult<T> = Result<PageHtml<T>, PageError>;
type ServerResult<T = ()> = Result<T, ServerError>;

type Attestation = Signature<Statement>;
type Token = Signature<Claims>;

#[cfg(feature = "fs")]
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
  #[expect(clippy::unnecessary_wraps)]
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

#[cfg(feature = "fs")]
pub fn run() -> ExitCode {
  CompleteEnv::with_factory(Arguments::command)
    .var("FILEPACK_COMPLETE")
    .complete();

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
