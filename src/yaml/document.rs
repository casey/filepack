use super::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Document {
  pub(crate) path: RelativePath,
}
