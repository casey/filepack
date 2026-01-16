use super::*;

pub(crate) struct Message {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) time: Option<u128>,
}

impl Message {
  pub(crate) fn digest(&self) -> Digest {
    let mut hasher = FingerprintHasher::new(FingerprintPrefix::Message);
    hasher.field(0, self.fingerprint.as_bytes());
    if let Some(time) = self.time {
      hasher.field(1, &time.to_le_bytes());
    }
    Digest(hasher.finalize())
  }
}
