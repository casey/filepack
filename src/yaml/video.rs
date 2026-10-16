use super::*;

#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Video {
  pub(crate) path: RelativePath,
  pub(crate) placeholder: Option<RelativePath>,
}
