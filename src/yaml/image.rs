use super::*;

#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Image {
  /// Path to the image file.
  #[json(example = "foo.png")]
  pub(crate) path: RelativePath,
}
