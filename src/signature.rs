use super::*;

#[derive(Clone, Copy, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub struct Signature {
  inner: ed25519_dalek::Signature,
  scheme: SignatureScheme,
}

impl Signature {
  pub(crate) const LEN: usize = ed25519_dalek::Signature::BYTE_SIZE;

  pub(crate) fn new(scheme: SignatureScheme, inner: ed25519_dalek::Signature) -> Self {
    Self { scheme, inner }
  }
}

impl AsRef<ed25519_dalek::Signature> for Signature {
  fn as_ref(&self) -> &ed25519_dalek::Signature {
    &self.inner
  }
}

impl Bech32m<1, { Signature::LEN }> for Signature {
  const HRP: Hrp = Hrp::parse_unchecked("signature");
  const TYPE: &'static str = "signature";
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Self::encode_bech32m(f, [self.scheme.into()], self.inner.to_bytes())
  }
}

impl fmt::Debug for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.inner, f)
  }
}

impl FromStr for Signature {
  type Err = SignatureError;

  fn from_str(signature: &str) -> Result<Self, Self::Err> {
    let ([scheme], data) = Self::decode_bech32m(signature)?.into_prefix_and_data();

    Ok(Self {
      inner: ed25519_dalek::Signature::from_bytes(&data),
      scheme: scheme.try_into()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let message = Message {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      time: None,
    };
    let signature = PrivateKey::generate().sign(&message.serialize());
    assert_eq!(
      signature.to_string().parse::<Signature>().unwrap(),
      signature
    );
  }
}
