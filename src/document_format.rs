use super::*;

/// Format of a document file.
#[derive(Clone, Copy, Debug, Decode, Display, Encode, Json, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum DocumentFormat {
  /// Portable Document Format.
  #[n(0)]
  Pdf,
}

impl Format for DocumentFormat {
  const EXTENSIONS: &[&str] = &["pdf"];

  fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "pdf" => Some(Self::Pdf),
      _ => None,
    }
  }

  fn resource_type(self) -> ResourceType {
    match self {
      Self::Pdf => ResourceType::Pdf,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(DocumentFormat::Pdf.to_string(), "PDF");
  }

  #[test]
  fn from_path() {
    #[track_caller]
    fn case(path: &str, expected: Result<DocumentFormat, PathError>) {
      assert_eq!(DocumentFormat::from_path(&path.parse().unwrap()), expected);
    }

    case("foo.pdf", Ok(DocumentFormat::Pdf));
    case(
      "foo.txt",
      Err(PathError::Extension {
        extensions: &["pdf"],
      }),
    );
    case(
      "foo",
      Err(PathError::Extension {
        extensions: &["pdf"],
      }),
    );
  }
}
