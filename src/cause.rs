use super::*;

pub(crate) enum Cause<'a> {
  Path(&'a RelativePath),
  Error(&'a dyn std::error::Error),
}

impl Display for Cause<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Path(path) => write!(f, "`{path}`"),
      Self::Error(error) => write!(f, "{error}"),
    }
  }
}
