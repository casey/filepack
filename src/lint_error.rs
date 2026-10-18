use super::*;

#[derive(Debug, EnumDiscriminants, PartialEq, Snafu)]
#[strum_discriminants(
  name(Lint),
  derive(
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    IntoStaticStr,
    Json,
    Ord,
    PartialOrd,
    Serialize
  ),
  vis(pub),
  serde(rename_all = "kebab-case"),
  strum(serialize_all = "kebab-case")
)]
pub(crate) enum LintError {
  /// Metadata has artwork.
  #[snafu(display("metadata missing artwork"))]
  ArtworkMissing,
  /// Audio files' embedded front cover art is square.
  #[snafu(display("audio file embedded front cover art is {dimensions} but must be square"))]
  AudioEmbeddedArtworkAspectRatio { dimensions: Dimensions },
  /// Audio files have embedded front cover art.
  #[snafu(display("audio file missing embedded front cover art"))]
  AudioEmbeddedArtworkMissing,
  /// Paths do not conflict on case-insensitive filesystems.
  #[snafu(display("paths would conflict on case-insensitive filesystem"))]
  CaseConflict,
  /// Metadata has a creator.
  #[snafu(display("metadata missing creator"))]
  CreatorMissing,
  /// Media items have titles.
  #[snafu(display("media item missing title"))]
  ItemTitleMissing,
  /// Package contains no junk files.
  #[snafu(display("possible junk file"))]
  Junk,
  /// Media has items.
  #[snafu(display("metadata media missing items"))]
  MediaItemsMissing,
  /// Metadata has media.
  #[snafu(display("metadata missing media"))]
  MediaMissing,
  /// Package has metadata.
  #[snafu(display("package missing metadata"))]
  MetadataMissing,
  /// Media items have the same format.
  #[snafu(display("media items have mixed formats: {}", formats.join(", ")))]
  MixedFormats { formats: Vec<String> },
  /// Derived assets have been generated.
  #[snafu(display("derived assets not generated, pass `--generate`"))]
  NotGenerated,
  /// Package metadata has a creator.
  #[snafu(display("metadata package missing creator"))]
  PackageCreatorMissing,
  /// Metadata has a package.
  #[snafu(display("metadata missing package"))]
  PackageMissing,
  /// Path components are at most 255 bytes.
  #[snafu(display("path component longer than 255 bytes"))]
  PathComponentLength,
  /// Metadata has a time.
  #[snafu(display("metadata missing time"))]
  TimeMissing,
  /// Metadata has a title.
  #[snafu(display("metadata missing title"))]
  TitleMissing,
  /// Placeholder images have the same resolution as their videos.
  #[snafu(display("placeholder image is {placeholder} but video is {video}"))]
  VideoPlaceholderDimensions {
    placeholder: Dimensions,
    video: Dimensions,
  },
  /// Videos have placeholder images.
  #[snafu(display("video missing placeholder image"))]
  VideoPlaceholderMissing,
  /// Filenames do not begin with a space, which Windows does not allow.
  #[snafu(display("Windows does not allow filenames that begin with spaces"))]
  WindowsLeadingSpace,
  /// Filenames do not contain characters Windows does not allow.
  #[snafu(display("Windows does not allow filenames that contain `{character}`"))]
  WindowsReservedCharacter { character: char },
  /// Filenames are not reserved by Windows.
  #[snafu(display("Windows does not allow files named `{name}`"))]
  WindowsReservedFilename { name: String },
  /// Filenames do not end with a period, which Windows does not allow.
  #[snafu(display("Windows does not allow filenames that end with a period"))]
  WindowsTrailingPeriod,
  /// Filenames do not end with a space, which Windows does not allow.
  #[snafu(display("Windows does not allow filenames that end with a space"))]
  WindowsTrailingSpace,
}
