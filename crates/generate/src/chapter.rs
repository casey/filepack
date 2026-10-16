use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Chapter {
  Json,
  Yaml,
}

impl Display for Chapter {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Json => "json",
      Self::Yaml => "yaml",
    }
    .fmt(f)
  }
}
