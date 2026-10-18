use super::*;

/// Video file media item.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Video {
  /// Path to the video file.
  #[json(example = "foo.mp4")]
  pub(crate) path: RelativePath,
  /// Path to a placeholder image. Must have the same resolution as the video file.
  #[json(example = "foo.png")]
  pub(crate) placeholder: Option<RelativePath>,
}
