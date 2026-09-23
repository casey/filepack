use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Ed25519Signature(ed25519_dalek::Signature);

impl Ed25519Signature {
  pub(crate) fn inner(self) -> ed25519_dalek::Signature {
    self.0
  }
}

impl Decode<'_> for Ed25519Signature {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    Ok(Self(ed25519_dalek::Signature::from_bytes(
      &decoder.byte_array()?,
    )))
  }
}

impl Encode for Ed25519Signature {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.bytes(&self.0.to_bytes());
  }
}

impl From<ed25519_dalek::Signature> for Ed25519Signature {
  fn from(inner: ed25519_dalek::Signature) -> Self {
    Self(inner)
  }
}

impl Ord for Ed25519Signature {
  fn cmp(&self, other: &Self) -> Ordering {
    self.0.to_bytes().cmp(&other.0.to_bytes())
  }
}

impl PartialOrd for Ed25519Signature {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
