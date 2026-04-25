use super::*;

#[derive(Clone, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
  #[n(0)]
  pub fingerprint: Fingerprint,
  #[n(1)]
  pub timestamp: Option<u64>,
}

impl Message {
  pub(crate) fn digest(&self) -> Hash {
    let envelope = Envelope {
      application: "filepack",
      ty: "message",
      message: self.clone(),
    };

    Hash::bytes(&envelope.encode_to_vec())
  }
}
