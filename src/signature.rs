use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(
  Debug, Decode, Encode, DeserializeFromStr, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay,
)]
#[deco(strict)]
pub struct Signature<T> {
  #[n(0)]
  version: Version,
  #[n(1)]
  public_key: PublicKey,
  #[n(2)]
  message: T,
  #[n(3)]
  signature: Ed25519Signature,
}

impl<T: Message> Signature<T> {
  pub(crate) fn new(
    public_key: PublicKey,
    message: T,
    signature: ed25519_dalek::Signature,
  ) -> Self {
    Self {
      version: Version::Zero,
      public_key,
      message,
      signature: signature.into(),
    }
  }

  pub fn public_key(&self) -> PublicKey {
    self.public_key
  }

  pub fn verify(&self, policy: T::Policy<'_>) -> Result<&T, T::Error> {
    self
      .public_key
      .inner()
      .verify_strict(
        self.message.digest(self.version).as_bytes(),
        &self.signature.inner(),
      )
      .context(signature_error::Invalid {
        public_key: self.public_key,
      })?;

    self.message.check(self.public_key, policy)?;

    Ok(&self.message)
  }
}

impl<T: Message> Display for Signature<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.format(f)
  }
}

impl<T: Message> FromStr for Signature<T> {
  type Err = HexError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl<T: Message> Hex for Signature<T> {
  const TAG: Tag = T::TAG;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn signature_begins_with_pubkey_and_fingerprint() {
    let prefix = format!(
      "signature1000001a0{}02a4000001a0{}03c0",
      &test::PUBLIC_KEY["public1".len()..],
      &test::FINGERPRINT["package1".len()..],
    );
    assert!(test::SIGNATURE.starts_with(&prefix));
    assert_eq!(test::SIGNATURE.len(), prefix.len() + 128);
  }

  #[test]
  fn unexpected_field_error() {
    let s = format!("{}0400", test::SIGNATURE);
    assert_matches!(
      s.parse::<Attestation>().unwrap_err(),
      HexError::Decode {
        source: DecodeError::UnknownField { key: 4 },
        tag: Tag::Signature,
      },
    );
  }
}
