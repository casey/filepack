use super::*;

/// Ed25519 signature over an enveloped message.
#[expect(clippy::arbitrary_source_item_ordering)]
#[derive(
  Debug, Decode, Encode, DeserializeFromStr, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay,
)]
#[deco(strict)]
pub struct Signature<T> {
  /// Public key that produced the signature.
  #[n(1)]
  public_key: PublicKey,
  /// Signed message.
  #[n(2)]
  message: T,
  /// Ed25519 signature over the enveloped message.
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
  const EXAMPLE: &'static str = T::EXAMPLE;
  const EXAMPLE_DESCRIPTION: &'static str = T::EXAMPLE_DESCRIPTION;
  const TAG: Tag = T::TAG;
}

impl<T: Message> json::Json for Signature<T> {
  fn json_schema(registry: &mut json::Registry) -> String {
    json::Schema::Hex {
      description: "Ed25519 signature over an enveloped message.".into(),
      tag: T::TAG,
    }
    .register(&format!("Signature<{}>", T::NAME), registry)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn signature_begins_with_pubkey_and_fingerprint() {
    let prefix = format!(
      "signature101a0{}02a401a0{}020003c0",
      &PublicKey::EXAMPLE["public1".len()..],
      &Fingerprint::EXAMPLE["package1".len()..],
    );
    assert!(Attestation::EXAMPLE.starts_with(&prefix));
    assert_eq!(Attestation::EXAMPLE.len(), prefix.len() + 128);
  }

  #[test]
  fn unexpected_field_error() {
    let s = format!("{}0400", Attestation::EXAMPLE);
    assert_matches!(
      s.parse::<Attestation>().unwrap_err(),
      HexError::Decode {
        source: DecodeError::Unknown {
          source: UnknownError::Field { key: 4 },
          strict: true
        },
        tag: Tag::Signature,
      },
    );
  }
}
