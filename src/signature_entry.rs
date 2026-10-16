use super::*;

#[derive(Json, Serialize)]
pub(crate) struct SignatureEntry {
  pub(crate) public_key: PublicKey,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) timestamp: Option<u64>,
}
