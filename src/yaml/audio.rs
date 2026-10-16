use super::*;

#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Audio {
  pub(crate) path: RelativePath,
}
