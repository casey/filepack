use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum VideoType {
  #[n(0)]
  Mp4,
}

impl VideoType {
  pub(crate) const EXTENSIONS: &[&str] = &["mp4"];

  pub(crate) fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "mp4" => Some(Self::Mp4),
      _ => None,
    }
  }

  pub(crate) fn resource_type(self) -> ResourceType {
    match self {
      Self::Mp4 => ResourceType::Mp4,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(VideoType::Mp4, "00");
  }

  #[test]
  fn from_extension() {
    assert_eq!(VideoType::from_extension("mp4"), Some(VideoType::Mp4));
    assert_eq!(VideoType::from_extension("avi"), None);
  }
}
