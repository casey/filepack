use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
  pub fingerprint: Fingerprint,
  pub time: Option<u128>,
}

impl Message {
  pub(crate) fn fingerprint(&self) -> Hash {
    let mut serializer = FingerprintHasher::new(FingerprintPrefix::Message);

    serializer.field(0, self.fingerprint.as_bytes());

    if let Some(time) = self.time {
      serializer.field(1, &time.to_le_bytes());
    }

    serializer.finalize()
  }
}
