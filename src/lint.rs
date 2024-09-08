use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Lint {
  FilenameLength,
  WindowsLeadingSpace,
  WindowsReservedCharacter { character: char },
  WindowsReservedFilename { name: String },
  WindowsTrailingPeriod,
  WindowsTrailingSpace,
}

impl Display for Lint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Lint::FilenameLength => write!(
        f,
        "Many filesystems do not allow filenames longer than 255 bytes"
      ),
      Lint::WindowsLeadingSpace => {
        write!(f, "Windows does not allow filenames that begin with spaces")
      }
      Lint::WindowsReservedCharacter { character } => write!(
        f,
        "Windows does not allow filenames that begin with `{character}`"
      ),
      Lint::WindowsReservedFilename { name } => {
        write!(f, "Windows does not allow files named `{name}`")
      }
      Lint::WindowsTrailingPeriod => {
        write!(f, "Windows does not allow filenames that end with a period")
      }
      Lint::WindowsTrailingSpace => {
        write!(f, "Windows does not allow filenmaes that end with a space")
      }
    }
  }
}

impl std::error::Error for Lint {}
