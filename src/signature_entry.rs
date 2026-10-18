use super::*;

/// Signature details.
#[derive(Json, Serialize)]
pub(crate) struct SignatureEntry {
  /// Public key derived from the private key that created the signature.
  pub(crate) public_key: PublicKey,
  /// Unix timestamp of signing in seconds. Set by the signer and not verifiable.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) timestamp: Option<u64>,
}
