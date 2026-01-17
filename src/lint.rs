use super::*;

#[derive(Debug, PartialEq, Snafu)]
pub(crate) enum Lint {
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
