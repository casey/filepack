use super::*;

#[derive(Debug, EnumDiscriminants, PartialEq, Snafu)]
#[strum_discriminants(
  name(Lint),
  derive(EnumIter, IntoStaticStr, Ord, PartialOrd, Serialize),
  serde(rename_all = "kebab-case"),
  strum(serialize_all = "kebab-case")
)]
pub(crate) enum LintError {
  #[snafu(display("filenames would confict on case-insensitive file system"))]
  CaseConflict,
  #[snafu(display("many filesystems do not allow filenames longer than 255 bytes"))]
  FilenameLength,
  #[snafu(display("possible junk file"))]
  Junk,
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
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for Lint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
