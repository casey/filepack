use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
  pub(crate) hash: Hash,
  pub(crate) size: u64,
}
