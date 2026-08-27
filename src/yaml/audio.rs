use super::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Audio {
  pub(crate) path: RelativePath,
}
