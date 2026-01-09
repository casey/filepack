use super::*;

#[derive(Debug, PartialEq)]
pub enum PathError {
  Component { component: String },
  ComponentEmpty,
  DoubleSlash,
  Empty,
  LeadingSlash,
  Separator { character: char },
  TrailingSlash,
  WindowsDiskPrefix { letter: char },
}

impl Display for PathError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Component { component } => write!(
        f,
        "paths may not contain non-normal path component `{component}`",
      ),
      Self::ComponentEmpty => write!(f, "paths may not contain empty components"),
      Self::DoubleSlash => write!(f, "paths may not contain double slashes"),
      Self::Empty => write!(f, "paths may not be empty"),
      Self::LeadingSlash => write!(f, "paths may not begin with slash character"),
      Self::Separator { character } => {
        write!(f, "paths may not contain separator character `{character}`")
      }
      Self::TrailingSlash => write!(f, "paths may not end with slash character"),
      Self::WindowsDiskPrefix { letter } => write!(
        f,
        "paths may not begin with Windows disk prefix `{letter}:`",
      ),
    }
  }
}

impl std::error::Error for PathError {}
