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
  fn display_is_lowercase_hex() {
    let s = "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
             0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3";
    assert_eq!(s.parse::<Signature>().unwrap().to_string(), s);
  }

  #[test]
  fn must_have_leading_zeros() {
    "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
     0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<Signature>()
      .unwrap();

    "f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
     0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<Signature>()
      .unwrap_err();
  }

  #[test]
  fn parse() {
    let signature = PrivateKey::generate().sign(Digest(test::HASH.parse().unwrap()));
    assert_eq!(
      signature.to_string().parse::<Signature>().unwrap(),
      signature
    );
  }

  #[test]
  fn uppercase_is_forbidden() {
    let signature = "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
                     0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3";
    signature.parse::<Signature>().unwrap();
    assert_eq!(
      signature
        .to_uppercase()
        .parse::<Signature>()
        .unwrap_err()
        .to_string(),
      "signatures must be lowercase hex: \
       `0F6D444F09EB336D3CC94D66CC541FEA0B70B36BE291EB3ECF5B49113F34C8D3\
        0F6D444F09EB336D3CC94D66CC541FEA0B70B36BE291EB3ECF5B49113F34C8D3`",
    );
  }
}
