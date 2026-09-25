use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServerState {
  pub number: u64,
  pub revision: Revision,
}
