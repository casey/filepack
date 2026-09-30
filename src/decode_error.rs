use super::*;

#[derive(Debug)]
pub enum DecodeError {
  Malformed(MalformedError),
  Unknown { source: UnknownError, strict: bool },
}

impl Display for DecodeError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Malformed(source) => Display::fmt(source, f),
      Self::Unknown { source, .. } => Display::fmt(source, f),
    }
  }
}

impl std::error::Error for DecodeError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Malformed(source) => source.source(),
      Self::Unknown { source, .. } => source.source(),
    }
  }
}

impl From<MalformedError> for DecodeError {
  fn from(source: MalformedError) -> Self {
    Self::Malformed(source)
  }
}
