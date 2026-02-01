use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
  pub fingerprint: Fingerprint,
  pub timestamp: Option<u64>,
}

impl Message {
  pub(crate) fn fingerprint(&self) -> Hash {
    let mut serializer = FingerprintHasher::new(FingerprintPrefix::Message);

    serializer.field(0, self.fingerprint.as_bytes());

    if let Some(timestamp) = self.timestamp {
      serializer.field(1, &timestamp.to_le_bytes());
    }

    serializer.finalize()
  }
}
