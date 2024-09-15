use super::*;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Entry {
  pub(crate) hash: Hash,
  pub(crate) size: u64,
}
