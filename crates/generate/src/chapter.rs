use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Chapter {
  Yaml,
}

impl Display for Chapter {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Yaml => "yaml",
    }
    .fmt(f)
  }
}
