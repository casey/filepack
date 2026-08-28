use super::*;

#[derive(Clone, Debug, Decode, Encode, PartialEq, Serialize)]
pub(crate) struct Document {
  #[n(0)]
  pub(crate) path: RelativePath,
  #[n(1)]
  #[serde(rename = "type")]
  pub(crate) ty: DocumentType,
}

impl Content for Document {
  const LABEL: &'static str = "Document";

  type Type = DocumentType;

  fn info(&self, builder: InfoBuilder) -> InfoBuilder {
    builder.value("type", self.ty)
  }

  fn load(_root: &Utf8Path, path: RelativePath) -> Result<Item<Self>> {
    let ty = DocumentType::from_path(&path).context(error::Path { path: &path })?;

    Ok(Item {
      content: Self { path, ty },
      title: None,
    })
  }

  fn path(&self) -> &RelativePath {
    &self.path
  }

  #[cfg(test)]
  fn test(path: &str) -> Self {
    let path = path.parse::<RelativePath>().unwrap();
    let ty = DocumentType::from_path(&path).unwrap();
    Self { path, ty }
  }

  fn ty(&self) -> Self::Type {
    self.ty
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
          path: "foo.pdf".parse().unwrap(),
          ty: DocumentType::Pdf,
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
      r#"{"path":"foo.pdf","type":"pdf"}"#,
    );
  }
}
