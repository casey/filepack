use super::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Video {
  pub(crate) cover: Option<RelativePath>,
  pub(crate) path: RelativePath,
}
