use super::*;

/// Properties of a text file.
#[derive(Clone, Debug, Decode, Encode, Eq, Json, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Text {
  /// Text file format.
  #[n(1)]
  #[serde(skip_serializing_if = "Decoded::is_unknown")]
  pub(crate) format: Decoded<TextFormat>,
  /// Path to the text file.
  #[n(2)]
  pub(crate) path: RelativePath,
}

impl Text {
  pub(crate) fn new(path: RelativePath) -> Result<Self> {
    let format = TextFormat::from_path(&path).context(error::Path { path: &path })?;
    Ok(Self {
      format: Decoded::Known(format),
      path,
    })
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self
      .format
      .known()
      .copied()
      .map_or(ResourceType::Binary, Format::resource_type)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn json() {
    assert_eq!(
      serde_json::to_string(&Text::new("foo.md".parse().unwrap()).unwrap()).unwrap(),
      r#"{"format":"markdown","path":"foo.md"}"#,
    );
  }

  #[test]
  fn new() {
    #[track_caller]
    fn case(path: &str, format: TextFormat) {
      assert_eq!(
        Text::new(path.parse().unwrap()).unwrap(),
        Text {
          format: Decoded::Known(format),
          path: path.parse().unwrap(),
        },
      );
    }

    case("foo.md", TextFormat::Markdown);
    case("foo.txt", TextFormat::Plain);
  }

  #[test]
  fn resource_type() {
    assert_eq!(
      Text::new("foo.md".parse().unwrap())
        .unwrap()
        .resource_type(),
      ResourceType::Text,
    );

    assert_eq!(
      Text {
        format: Decoded::Unknown(vec![99]),
        path: "foo".parse().unwrap(),
      }
      .resource_type(),
      ResourceType::Binary,
    );
  }
}
