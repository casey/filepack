use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
  Character { character: char },
  Component { component: String },
  DoubleSlash,
  Empty,
  LeadingSlash,
  TrailingSlash,
  WindowsDiskPrefix { letter: char },
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Error::Character { character } => write!(f, "illegal character `{character}`"),
      Error::Component { component } => write!(f, "illegal path component `{component}`"),
      Error::DoubleSlash => write!(f, "double slash"),
      Error::Empty => write!(f, "empty path"),
      Error::LeadingSlash => write!(f, "leading slash"),
      Error::TrailingSlash => write!(f, "trailing slash"),
      Error::WindowsDiskPrefix { letter } => write!(f, "Windows disk prefix `{letter}:`"),
    }
  }
}

impl std::error::Error for Error {}
