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
    archive_builder::ArchiveBuilder,
    archive_error::ArchiveError,
    arguments::Arguments,
    array_decoder::ArrayDecoder,
    array_encoder::ArrayEncoder,
    audio::Audio,
    audio_error::AudioError,
    audio_metadata::AudioMetadata,
    audio_position_error::AudioPositionError,
    audio_type::AudioType,
    authenticated::Authenticated,
    authorization_error::AuthorizationError,
    bit_reader::BitReader,
    cause::Cause,
    checked_url::CheckedUrl,
    chroma_subsampling::ChromaSubsampling,
    claims::Claims,
    client::Client,
    codec::Codec,
    color_info::ColorInfo,
    color_type::ColorType,
    component::Component,
    component_buf::ComponentBuf,
    component_error::ComponentError,
    compression::Compression,
    content::Content,
    content_type::ContentType,
    context::Context,
    count::Count,
    database_metadata::DatabaseMetadata,
    deco_request::DecoRequest,
    deco_response::DecoResponse,
    decode_error::DecodeError,
    decode_options::DecodeOptions,
    decode_owned::DecodeOwned,
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
    ed25519_signature::Ed25519Signature,
    embedded_image::EmbeddedImage,
    entries::Entries,
    entry::EntryType,
    envelope::Envelope,
    error::Error,
    exif_decoder::ExifDecoder,
    exif_error::ExifError,
    file::File,
    filesystem_error::FilesystemError,
    flac_decoder::FlacDecoder,
    float_ext::FloatExt,
    format::Format,
    functions::{
      current_dir, decode_path, default, format_size, ignore, now, parse_number, transfer_tempfile,
    },
    hashing_writer::HashingWriter,
    head::Head,
    hex::Hex,
    hex_error::HexError,
    image::Image,
    image_metadata::ImageMetadata,
    image_type::ImageType,
    info::Info,
    info_builder::InfoBuilder,
    invalid_public_key::InvalidPublicKey,
    iso8601_duration::Iso8601Duration,
    item::Item,
    key_identifier::KeyIdentifier,
    key_identifier_error::KeyIdentifierError,
    key_name::KeyName,
    key_type::KeyType,
    keychain::Keychain,
    language::Language,
    language_error::LanguageError,
    lint_error::{Lint, LintError},
    lint_group::LintGroup,
    lint_selector::LintSelector,
    linter::Linter,
    map_decoder::MapDecoder,
    map_encoder::MapEncoder,
    media::{Media, MediaType},
    media_item::MediaItem,
    media_item_resource::MediaItemResource,
    message::Message,
    mode::Mode,
    mp3_decoder::Mp3Decoder,
    mp3_error::Mp3Error,
    mp4_decoder::Mp4Decoder,
    number_error::NumberError,
    open_graph_image::OpenGraphImage,
    options::Options,
    or::Or,
    or_unknown::OrUnknown,
    order::Order,
    ordinal::Ordinal,
    orientation::Orientation,
    owo_colorize_ext::OwoColorizeExt,
    package::Package,
    package_identifier_error::PackageIdentifierError,
    package_summary::PackageSummary,
    page_error::PageError,
    path_error::PathError,
    percent_encode::PercentEncode,
    private_key_error::PrivateKeyError,
    progress_bar::ProgressBar,
    public_key_error::PublicKeyError,
    relative_path::RelativePath,
    reqwest_response_ext::ReqwestResponseExt,
    resolved::Resolved,
    resource::Resource,
    resource_type::ResourceType,
    rotation::Rotation,
    server::Server,
    server_error::ServerError,
    server_url::ServerUrl,
    sign_options::SignOptions,
    signature::Signature,
    signature_error::SignatureError,
    sort::Sort,
    sort_key::SortKey,
    sorted_set::SortedSet,
    statement::Statement,
    static_asset::StaticAsset,
    style::Style,
    subcommand::Subcommand,
    tag::Tag,
    templates::{ErrorHtml, PageHtml},
    text::Text,
    text_error::TextError,
    ticked::Ticked,
    time::Time,
    time_error::TimeError,
    totals_error::TotalsError,
    track::Track,
    track_info::TrackInfo,
    type_name::TypeName,
    url_error::UrlError,
    utf8_path_ext::Utf8PathExt,
    validate::Validate,
    video::Video,
    video_error::VideoError,
    video_metadata::VideoMetadata,
    video_type::VideoType,
    webm_decoder::WebmDecoder,
    xmp_error::XmpError,
  },
  axum::{
    body::Body,
    http::{self, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
  },
  axum_extra::{TypedHeader, headers},
  blake3::Hasher,
  boilerplate::{Boilerplate, Trusted},
  camino::{Utf8Component, Utf8Path, Utf8PathBuf},
  clap::{ArgGroup, Parser, ValueEnum},
  claxon::FlacReader,
  filepack_derive::{Decode, DecodeFromStr, Encode, EncodeDisplay, Magic},
  futures_util::StreamExt,
  humansize::{BINARY, BaseUnit, DECIMAL, FormatSizeOptions, SizeFormatter},
  id3::TagLike,
  jiff::{self, civil},
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
    env,
    fmt::{self, Debug, Display, Formatter},
    fs::{self, Permissions},
    io::{self, BufReader, IsTerminal, Read, Seek, SeekFrom, Write},
    iter, mem,
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
    vec,
  },
  strum::{
    Display, EnumDiscriminants, EnumIter, EnumString, FromRepr, IntoDiscriminant, IntoEnumIterator,
    IntoStaticStr, VariantArray,
  },
  tempfile::NamedTempFile,
  tokio::io::{AsyncReadExt, AsyncWriteExt},
  tokio_util::io::ReaderStream,
  unicase::UniCase,
  url::{Host, Url},
  usized::IntoU64,
  walkdir::WalkDir,
  zune_jpeg::{JpegDecoder, SampleRatios, zune_core::colorspace::ColorSpace},
};

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
  unindent::unindent,
  webm_builder::WebmBuilder,
};

pub use self::{
  archive::Archive,
  decode::Decode,
  decoder::Decoder,
  directory::Directory,
  directory_ext::DirectoryExt,
  encode::Encode,
  encoder::Encoder,
  entry::Entry,
  fingerprint::Fingerprint,
  flac_builder::FlacBuilder,
  functions::{gradient, gradient_alpha, install_default_crypto_provider},
  hash::Hash,
  loader::Loader,
  magic::Magic,
  magic_type::MagicType,
  manifest::Manifest,
  metadata::Metadata,
  mp3_builder::Mp3Builder,
  mp4_builder::Mp4Builder,
  package_identifier::PackageIdentifier,
  page::Page,
  png_builder::PngBuilder,
  private_key::PrivateKey,
  public_key::PublicKey,
  revision::Revision,
  revision_object::RevisionObject,
  server_state::ServerState,
  state::State,
  totals::Totals,
  version::Version,
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
mod arguments;
mod array_decoder;
mod array_encoder;
mod audio;
mod audio_error;
mod audio_metadata;
mod audio_position_error;
mod audio_type;
mod authenticated;
mod authorization_error;
mod bit_reader;
mod cause;
mod checked_url;
mod chroma_subsampling;
mod claims;
mod client;
mod codec;
mod color_info;
mod color_type;
mod component;
mod component_buf;
mod component_error;
mod compression;
mod content;
mod content_type;
mod context;
mod count;
mod database_metadata;
mod deco_request;
mod deco_response;
mod decode;
mod decode_error;
mod decode_options;
mod decode_owned;
mod decoder;
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
mod ed25519_signature;
mod embedded_image;
mod encode;
mod encoder;
mod entries;
mod entry;
mod envelope;
mod error;
mod exif_decoder;
mod exif_error;
mod file;
mod filesystem;
mod filesystem_error;
mod fingerprint;
mod flac_builder;
mod flac_decoder;
mod float_ext;
mod format;
mod functions;
mod hash;
mod hashing_writer;
mod head;
pub mod hex;
mod hex_error;
mod image;
mod image_metadata;
mod image_type;
mod info;
mod info_builder;
mod invalid_public_key;
mod iso8601_duration;
mod item;
mod key_identifier;
mod key_identifier_error;
mod key_name;
mod key_type;
mod keychain;
mod language;
mod language_error;
mod lint_error;
mod lint_group;
mod lint_selector;
mod linter;
mod loader;
mod magic;
mod magic_type;
mod manifest;
mod map_decoder;
mod map_encoder;
mod media;
mod media_item;
mod media_item_resource;
mod message;
mod metadata;
mod mode;
mod mp3_builder;
mod mp3_decoder;
mod mp3_error;
mod mp4_builder;
mod mp4_decoder;
mod number_error;
mod open_graph_image;
mod options;
mod or;
mod or_unknown;
mod order;
mod ordinal;
mod orientation;
mod owo_colorize_ext;
mod package;
mod package_identifier;
mod package_identifier_error;
mod package_summary;
mod page;
mod page_error;
mod path_error;
mod percent_encode;
mod png_builder;
mod private_key;
mod private_key_error;
mod progress_bar;
mod public_key;
mod public_key_error;
mod re;
mod relative_path;
mod reqwest_response_ext;
mod resolved;
mod resource;
mod resource_type;
mod revision;
mod revision_object;
mod rotation;
mod server;
mod server_error;
mod server_state;
mod server_url;
mod sign_options;
mod signature;
mod signature_error;
mod sort;
mod sort_key;
mod sorted_set;
mod state;
mod statement;
mod static_asset;
mod style;
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
mod track_info;
mod type_name;
mod url_error;
mod utf8_path_ext;
mod validate;
mod version;
mod video;
mod video_error;
mod video_metadata;
mod video_type;
mod webm_decoder;
mod xmp;
mod xmp_error;
mod yaml;

#[cfg(test)]
mod derive;
#[cfg(test)]
mod jpeg_builder;
#[cfg(test)]
mod test;
#[cfg(test)]
mod webm_builder;

const KIB: usize = 1 << 10;
const MIB: usize = KIB << 10;

type Result<T = (), E = Error> = std::result::Result<T, E>;

type DecodeResult<T = ()> = Result<T, DecodeError>;
type PageResult<T> = Result<PageHtml<T>, PageError>;
type ServerResult<T = ()> = Result<T, ServerError>;

type Attestation = Signature<Statement>;
type Token = Signature<Claims>;

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
