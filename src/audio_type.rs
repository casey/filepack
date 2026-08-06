use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AudioType {
  #[n(0)]
  #[strum(serialize = "FLAC")]
  Flac,
  #[n(1)]
  #[strum(serialize = "MP3")]
  Mp3,
}

impl AudioType {
  pub(crate) const EXTENSIONS: &[&str] = &["flac", "mp3"];

  pub(crate) fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "flac" => Some(Self::Flac),
      "mp3" => Some(Self::Mp3),
      _ => None,
    }
  }

  pub(crate) fn resource_type(self) -> ResourceType {
    match self {
      Self::Flac => ResourceType::Flac,
      Self::Mp3 => ResourceType::Mp3,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(AudioType::Flac.to_string(), "FLAC");
    assert_eq!(AudioType::Mp3.to_string(), "MP3");
  }

  #[test]
  fn from_extension() {
    assert_eq!(AudioType::from_extension("flac"), Some(AudioType::Flac));
    assert_eq!(AudioType::from_extension("mp3"), Some(AudioType::Mp3));
    assert_eq!(AudioType::from_extension("wav"), None);
  }
}
