use super::*;

#[derive(Clone, Copy, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub struct Signature(ed25519_dalek::Signature);

impl Signature {
  pub(crate) const LEN: usize = ed25519_dalek::Signature::BYTE_SIZE;
}

impl AsRef<ed25519_dalek::Signature> for Signature {
  fn as_ref(&self) -> &ed25519_dalek::Signature {
    &self.0
  }
}

impl Bech32m<{ Signature::LEN }> for Signature {
  const HRP: Hrp = Hrp::parse_unchecked("signature");
}

impl From<ed25519_dalek::Signature> for Signature {
  fn from(inner: ed25519_dalek::Signature) -> Self {
    Self(inner)
  }
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Self::encode_bech32m(f, self.0.to_bytes())
  }
}

impl fmt::Debug for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.0, f)
  }
}

impl FromStr for Signature {
  type Err = Bech32mError;

  fn from_str(signature: &str) -> Result<Self, Self::Err> {
    let bytes = Self::decode_bech32m(signature)?;
    Ok(Self(ed25519_dalek::Signature::from_bytes(&bytes)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let signature = PrivateKey::generate().sign(Digest(test::HASH.parse().unwrap()));
    assert_eq!(
      signature.to_string().parse::<Signature>().unwrap(),
      signature
    );
  }
}
