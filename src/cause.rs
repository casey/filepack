use super::*;

pub(crate) enum Cause<'a> {
  Error(&'a dyn std::error::Error),
  Path(&'a RelativePath),
}

impl Display for Cause<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Error(error) => write!(f, "{error}"),
      Self::Path(path) => write!(f, "`{path}`"),
    }
  }
}
