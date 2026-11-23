use super::*;

/// Format of a text file.
#[derive(
  Clone, Copy, Debug, Decode, Display, Encode, Eq, Json, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum TextFormat {
  /// [Markdown](https://en.wikipedia.org/wiki/Markdown).
  #[n(0)]
  Markdown,
  /// Plain text.
  #[n(1)]
  Plain,
}

impl Format for TextFormat {
  const EXTENSIONS: &[&str] = &["md", "txt"];

  fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "md" => Some(Self::Markdown),
      "txt" => Some(Self::Plain),
      _ => None,
    }
  }

  fn resource_type(self) -> ResourceType {
    ResourceType::Text
  }
}
