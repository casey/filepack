use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Debug, Decode, Encode, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Signature<T> {
  #[n(0)]
  public_key: PublicKey,
  #[n(1)]
  message: T,
  #[n(2)]
  signature: Ed25519Signature,
}

impl<T: Message> Signature<T> {
  fn comparison_key(&self) -> (PublicKey, &T, [u8; 64]) {
    (
      self.public_key,
      &self.message,
      self.signature.inner().to_bytes(),
    )
  }

  pub(crate) fn new(
    public_key: PublicKey,
    message: T,
    signature: ed25519_dalek::Signature,
  ) -> Self {
    Self {
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
      .verify_strict(self.message.digest().as_bytes(), &self.signature.inner())
      .context(signature_error::Invalid {
        public_key: self.public_key,
      })?;

    self.message.check(self.public_key, policy)?;

    Ok(&self.message)
  }
}

impl<T: Message> Ord for Signature<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.comparison_key().cmp(&other.comparison_key())
  }
}

impl<T: Message> PartialOrd for Signature<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<T: Message> Hex for Signature<T> {
  const TAG: Tag = T::TAG;
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn signature_begins_with_pubkey_and_fingerprint() {
    let prefix = format!(
      "signature100a0{}01a200a0{}02c0",
      &test::PUBLIC_KEY["public1".len()..],
      &test::FINGERPRINT["package1".len()..],
    );
    assert!(test::SIGNATURE.starts_with(&prefix));
    assert_eq!(test::SIGNATURE.len(), prefix.len() + 128);
  }

  #[test]
  fn unexpected_field_error() {
    let s = format!("{}0300", test::SIGNATURE);
    assert_matches!(
      s.parse::<Attestation>().unwrap_err(),
      HexError::Decode {
        source: DecodeError::UnconsumedEntries,
        tag: Tag::Signature,
      },
    );
  }
}
