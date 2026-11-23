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
}
