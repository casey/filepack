use super::*;

pub(crate) struct Message {
  pub(crate) fingerprint: Hash,
}

impl Message {
  pub(crate) fn digest(self) -> Hash {
    let mut hasher = FingerprintHasher::new(FingerprintPrefix::Message);
    hasher.field(0, self.fingerprint.as_bytes());
    hasher.finalize()
  }
}
