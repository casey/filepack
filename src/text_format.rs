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

  fn extension(self) -> &'static str {
    match self {
      Self::Markdown => "md",
      Self::Plain => "txt",
    }
  }

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extension() {
    #[track_caller]
    fn case(format: TextFormat, extension: &str) {
      assert_eq!(format.extension(), extension);
      assert_eq!(TextFormat::from_extension(extension), Some(format));
    }

    case(TextFormat::Markdown, "md");
    case(TextFormat::Plain, "txt");
  }
}
