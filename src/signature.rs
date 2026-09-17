use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Debug, Decode, Encode, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Signature {
  #[n(0)]
  public_key: PublicKey,
  #[n(1)]
  statement: Statement,
  #[n(2)]
  #[deco(decode_with = Signature::decode_signature, encode_with = Signature::encode_signature)]
  signature: ed25519_dalek::Signature,
}

impl Signature {
  fn comparison_key(&self) -> (PublicKey, &Statement, [u8; 64]) {
    (self.public_key, &self.statement, self.signature.to_bytes())
  }

  fn decode_signature(decoder: &mut Decoder) -> Result<ed25519_dalek::Signature, DecodeError> {
    Ok(ed25519_dalek::Signature::from_bytes(&decoder.byte_array()?))
  }

  fn encode_signature(signature: &ed25519_dalek::Signature, encoder: &mut Encoder) {
    encoder.bytes(&signature.to_bytes());
  }

  pub(crate) fn new(
    public_key: PublicKey,
    statement: Statement,
    signature: ed25519_dalek::Signature,
  ) -> Self {
    Self {
      public_key,
      statement,
      signature,
    }
  }

  pub fn public_key(&self) -> PublicKey {
    self.public_key
  }

  pub fn statement(&self) -> &Statement {
    &self.statement
  }

  pub(crate) fn verify(&self, fingerprint: Fingerprint) -> Result {
    ensure! {
      fingerprint == self.statement.fingerprint,
      error::SignatureFingerprintMismatch {
        signature: self.statement.fingerprint,
        package: fingerprint,
      },
    }

    self
      .public_key
      .inner()
      .verify_strict(self.statement.digest().as_bytes(), &self.signature)
      .map_err(DalekSignatureError)
      .context(error::SignatureInvalid {
        public_key: self.public_key,
      })
  }
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.format(f)
  }
}

impl FromStr for Signature {
  type Err = HexError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl Hex for Signature {
  const TAG: Tag = Tag::Signature;
}

impl Ord for Signature {
  fn cmp(&self, other: &Self) -> Ordering {
    self.comparison_key().cmp(&other.comparison_key())
  }
}

impl PartialOrd for Signature {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn modifying_fingerprint_invalidates_signature() {
    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();
    let statement = Statement {
      fingerprint,
      timestamp: Some(1000),
    };
    let mut signature = private_key.sign(&statement);
    signature.statement.fingerprint = Fingerprint::from_bytes(default());
    assert_matches!(
      signature.verify(fingerprint).unwrap_err(),
      Error::SignatureFingerprintMismatch { .. },
    );
  }

  #[test]
  fn modifying_time_invalidates_signature() {
    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();
    let statement = Statement {
      fingerprint,
      timestamp: Some(1000),
    };
    let mut signature = private_key.sign(&statement);
    signature.statement.timestamp = Some(2000);
    assert_matches!(
      signature.verify(fingerprint).unwrap_err(),
      Error::SignatureInvalid { .. },
    );
  }

  #[test]
  fn removing_time_invalidates_signature() {
    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();
    let statement = Statement {
      fingerprint,
      timestamp: Some(1000),
    };
    let mut signature = private_key.sign(&statement);
    signature.statement.timestamp = None;
    assert_matches!(
      signature.verify(fingerprint).unwrap_err(),
      Error::SignatureInvalid { .. },
    );
  }

  #[test]
  fn signature_begins_with_pubkey_and_fingerprint() {
    let prefix = format!(
      "signature1f08800{}01a200{}02c0",
      &test::PUBLIC_KEY["public1".len()..],
      &test::FINGERPRINT["package1".len()..],
    );
    assert!(test::SIGNATURE.starts_with(&prefix));
    assert_eq!(test::SIGNATURE.len(), prefix.len() + 128);
  }

  #[test]
  fn unexpected_field_error() {
    let s = format!(
      "signature1f08a{}0300",
      &test::SIGNATURE["signature1f088".len()..]
    );
    assert_matches!(
      s.parse::<Signature>().unwrap_err(),
      HexError::Decode {
        source: DecodeError::UnconsumedEntries,
        tag: Tag::Signature,
      },
    );
  }
}
