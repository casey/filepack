use super::*;

#[derive(Debug, EnumDiscriminants, PartialEq, Snafu)]
#[strum_discriminants(name(Lint), derive(Ord, PartialOrd))]
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
