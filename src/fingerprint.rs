use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Fingerprint(pub(crate) Hash);

impl Fingerprint {
  pub(crate) const LEN: usize = Hash::LEN;

  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }

  pub(crate) fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
    Self(bytes.into())
  }
}

impl Display for Fingerprint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32Encoder::new(Bech32Type::Fingerprint);
    encoder.bytes(self.as_bytes());
    write!(f, "{encoder}")
  }
}

#[cfg(test)]
impl Decode for Fingerprint {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    Ok(Self(Hash::decode(decoder)?))
  }
}

impl Encode for Fingerprint {
  fn encode(&self, encoder: &mut Encoder) {
    self.0.encode(encoder);
  }
}

impl FromStr for Fingerprint {
  type Err = Bech32Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let inner = Bech32Decoder::decode_byte_array(Bech32Type::Fingerprint, s)?;
    Ok(Self(inner.into()))
  }
}
