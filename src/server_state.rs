use super::*;

/// State entry for a server.
#[derive(Debug, Deserialize, Json, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServerState {
  /// Package number on the server.
  pub number: u64,
  /// Head of the package number when last uploaded or downloaded.
  pub revision: RevisionHash,
}
