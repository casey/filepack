use super::*;

pub(crate) struct MessageFingerprint(pub(crate) Hash);

impl MessageFingerprint {
  pub(crate) fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }
}
