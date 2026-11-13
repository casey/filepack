use super::*;

/// Properties of a document file.
#[derive(Clone, Debug, Decode, Encode, Json, PartialEq, Serialize)]
pub(crate) struct Document {
  /// Document file format.
  #[n(1)]
  #[serde(skip_serializing_if = "Decoded::is_unknown")]
  pub(crate) format: Decoded<DocumentFormat>,
  /// Path to the document file.
  #[n(2)]
  pub(crate) path: RelativePath,
}

impl Content for Document {
  const LABEL: &'static str = "Document";

  type Format = DocumentFormat;

  fn format(&self) -> &Decoded<Self::Format> {
    &self.format
  }

  fn info(&self, builder: InfoBuilder) -> InfoBuilder {
    builder.value("format", &self.format)
  }

  fn load(_root: &Utf8Path, path: RelativePath) -> Result<Item<Self>> {
    let format = DocumentFormat::from_path(&path).context(error::Path { path: &path })?;

    Ok(Item {
      content: Self {
        format: Decoded::Known(format),
        path,
      },
      title: None,
    })
  }

  fn path(&self) -> &RelativePath {
    &self.path
  }

  #[cfg(test)]
  fn test(path: &str) -> Self {
    let path = path.parse::<RelativePath>().unwrap();
    let format = DocumentFormat::from_path(&path).unwrap();
    Self {
      format: Decoded::Known(format),
      path,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn load() {
    let (_tempdir, root) = tempdir();

    std::fs::write(root.join("foo.pdf"), b"%PDF-1.7\n").unwrap();

    assert_eq!(
      Document::load(&root, "foo.pdf".parse().unwrap()).unwrap(),
      Item {
        content: Document {
          format: Decoded::Known(DocumentFormat::Pdf),
          path: "foo.pdf".parse().unwrap(),
        },
        title: None,
      },
    );
  }

  #[test]
  fn load_rejects_invalid_extension() {
    let (_tempdir, root) = tempdir();

    assert_eq!(
      Document::load(&root, "foo.txt".parse().unwrap())
        .unwrap_err()
        .to_string(),
      "invalid path `foo.txt`",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Document::test("foo.pdf")).unwrap(),
      r#"{"format":"pdf","path":"foo.pdf"}"#,
    );
  }
}
