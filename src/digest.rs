use super::*;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Digest(pub(crate) Fingerprint);

impl Digest {
  #[must_use]
  pub fn as_bytes(&self) -> &[u8; Hash::LEN] {
    self.0.as_bytes()
  }
}
