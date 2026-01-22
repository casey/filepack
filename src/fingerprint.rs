use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Fingerprint(pub(crate) Hash);

impl Fingerprint {
  pub(crate) const LEN: usize = Hash::LEN;

  #[must_use]
  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }
}

impl Bech32m<0, { Fingerprint::LEN }> for Fingerprint {
  const TYPE: Bech32mType = Bech32mType::Fingerprint;
  type Suffix = ();
}

impl Display for Fingerprint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Self::encode_bech32m(f, Bech32mPayload::from_body(*self.as_bytes()))
  }
}

impl FromStr for Fingerprint {
  type Err = Bech32mError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(Self::decode_bech32m(s)?.into_body().into()))
  }
}
