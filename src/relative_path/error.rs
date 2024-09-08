use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
  Component { component: String },
  DoubleSlash,
  Empty,
  LeadingSlash,
  Separator { character: char },
  TrailingSlash,
  WindowsDiskPrefix { letter: char },
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Error::Component { component } => write!(
        f,
        "paths may not contain non-normal path component `{component}`"
      ),
      Error::DoubleSlash => write!(f, "paths may not contain double slashes"),
      Error::Empty => write!(f, "paths may not be empty"),
      Error::LeadingSlash => write!(f, "paths may not begin with slash character"),
      Error::Separator { character } => {
        write!(f, "paths may not contain separator character `{character}`")
      }
      Error::TrailingSlash => write!(f, "paths may not end with slash character"),
      Error::WindowsDiskPrefix { letter } => write!(
        f,
        "paths may not begin with Windows disk prefix `{letter}:`"
      ),
    }
  }
}

impl std::error::Error for Error {}
