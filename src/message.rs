use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Message {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) time: Option<u128>,
}

impl Message {
  pub(crate) fn serialize(&self) -> SerializedMessage {
    let mut serializer =
      FingerprintSerializer::new(FingerprintPrefix::Message, Vec::new()).unwrap();

    serializer.field(0, self.fingerprint.as_bytes()).unwrap();

    if let Some(time) = self.time {
      serializer.field(1, &time.to_le_bytes()).unwrap();
    }

    SerializedMessage(serializer.into_inner())
  }
}
