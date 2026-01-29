use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Fingerprint(pub(crate) Hash);

impl Fingerprint {
  pub(crate) const LEN: usize = Hash::LEN;

  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }
}

impl Display for Fingerprint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32Encoder::new(Bech32Type::Fingerprint);
    encoder.bytes(self.as_bytes());
    write!(f, "{encoder}")
  }
}

impl FromStr for Fingerprint {
  type Err = Bech32Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let inner = Bech32Decoder::decode_byte_array(Bech32Type::Fingerprint, s)?;
    Ok(Self(inner.into()))
  }
}
