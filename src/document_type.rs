use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum DocumentType {
  #[n(0)]
  Pdf,
}

impl ContentType for DocumentType {
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
    assert_eq!(DocumentType::Pdf.to_string(), "PDF");
  }

  #[test]
  fn from_path() {
    #[track_caller]
    fn case(path: &str, expected: Result<DocumentType, PathError>) {
      assert_eq!(DocumentType::from_path(&path.parse().unwrap()), expected);
    }

    case("foo.pdf", Ok(DocumentType::Pdf));
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
