use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum ImageType {
  #[n(0)]
  Jpeg,
  #[n(1)]
  Png,
}

impl ImageType {
  pub(crate) const EXTENSIONS: &[&str] = &["jpg", "png"];

  pub(crate) fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "jpg" => Some(Self::Jpeg),
      "png" => Some(Self::Png),
      _ => None,
    }
  }

  pub(crate) fn resource_type(self) -> ResourceType {
    match self {
      Self::Jpeg => ResourceType::Jpeg,
      Self::Png => ResourceType::Png,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(ImageType::Jpeg, "00");
    assert_cbor(ImageType::Png, "01");
  }

  #[test]
  fn from_extension() {
    assert_eq!(ImageType::from_extension("jpg"), Some(ImageType::Jpeg));
    assert_eq!(ImageType::from_extension("png"), Some(ImageType::Png));
    assert_eq!(ImageType::from_extension("svg"), None);
  }
}
