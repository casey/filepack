use super::*;

/// Document file media item.
#[derive(Debug, Deserialize, Json, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Document {
  /// Path to the document file.
  #[json(example = "foo.pdf")]
  pub(crate) path: RelativePath,
}
