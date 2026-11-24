use super::*;

/// Text file media item.
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Text {
  /// Path to the text file.
  #[json(example = "foo.md")]
  pub(crate) path: RelativePath,
  /// Title of the text file.
  #[json(example = "Foo")]
  pub(crate) title: Line,
}
