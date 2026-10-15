use super::*;

#[derive(Debug, Deserialize, PartialEq, Yaml)]
#[serde(deny_unknown_fields)]
pub(crate) struct Audio {
  pub(crate) path: RelativePath,
}
