use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AudioType {
  #[n(0)]
  Flac,
}

impl Display for AudioType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Flac => write!(f, "FLAC"),
    }
  }
}

impl AudioType {
  pub(crate) const EXTENSIONS: &[&str] = &["flac"];

  pub(crate) fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "flac" => Some(Self::Flac),
      _ => None,
    }
  }

  pub(crate) fn resource_type(self) -> ResourceType {
    match self {
      Self::Flac => ResourceType::Flac,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(AudioType::Flac.to_string(), "FLAC");
  }

  #[test]
  fn encoding() {
    assert_cbor(AudioType::Flac, "00");
  }

  #[test]
  fn from_extension() {
    assert_eq!(AudioType::from_extension("flac"), Some(AudioType::Flac));
    assert_eq!(AudioType::from_extension("mp3"), None);
  }
}
