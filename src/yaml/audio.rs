use super::*;

#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Audio {
  /// Path to the audio file.
  #[json(example = "foo.flac")]
  pub(crate) path: RelativePath,
}
