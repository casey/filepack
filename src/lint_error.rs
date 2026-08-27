use super::*;

#[derive(Debug, EnumDiscriminants, PartialEq, Snafu)]
#[strum_discriminants(
  name(Lint),
  derive(EnumIter, EnumString, IntoStaticStr, Ord, PartialOrd, Serialize),
  serde(rename_all = "kebab-case"),
  strum(serialize_all = "kebab-case")
)]
pub(crate) enum LintError {
  #[snafu(display("metadata missing artwork"))]
  ArtworkMissing,
  #[snafu(display("audio file missing embedded front cover art"))]
  AudioEmbeddedArtworkMissing,
  #[snafu(display("paths would conflict on case-insensitive filesystem"))]
  CaseConflict,
  #[snafu(display("metadata missing creator"))]
  CreatorMissing,
  #[snafu(display("possible junk file"))]
  Junk,
  #[snafu(display("metadata media missing items"))]
  MediaItemsMissing,
  #[snafu(display("metadata missing media"))]
  MediaMissing,
  #[snafu(display("package missing metadata"))]
  MetadataMissing,
  #[snafu(display("derived assets not generated, pass `--generate`"))]
  NotGenerated,
  #[snafu(display("metadata package missing creator"))]
  PackageCreatorMissing,
  #[snafu(display("metadata missing package"))]
  PackageMissing,
  #[snafu(display("metadata missing time"))]
  TimeMissing,
  #[snafu(display("metadata missing title"))]
  TitleMissing,
  #[snafu(display("video missing placeholder image"))]
  VideoPlaceholderMissing,
  #[snafu(display("Windows does not allow filenames that begin with spaces"))]
  WindowsLeadingSpace,
  #[snafu(display("Windows does not allow filenames that begin with `{character}`"))]
  WindowsReservedCharacter { character: char },
  #[snafu(display("Windows does not allow files named `{name}`"))]
  WindowsReservedFilename { name: String },
  #[snafu(display("Windows does not allow filenames that end with a period"))]
  WindowsTrailingPeriod,
  #[snafu(display("Windows does not allow filenames that end with a space"))]
  WindowsTrailingSpace,
}

impl Lint {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for Lint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
