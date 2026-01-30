use super::*;

pub(crate) struct Digest(pub(crate) Hash);

impl Digest {
  pub(crate) fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }
}
