use super::*;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct File {
  pub hash: Hash,
  pub size: u64,
}

impl File {
  pub(crate) fn fingerprint(&self) -> Hash {
    let mut hasher = FingerprintHasher::new(Context::File);
    hasher.field(0, Hash::bytes(&self.size.to_le_bytes()));
    hasher.field(1, self.hash);
    hasher.finalize()
  }
}
