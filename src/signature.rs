use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Signature {
  message: Message,
  public_key: PublicKey,
  signature: ed25519_dalek::Signature,
}

impl Signature {
  fn comparison_key(&self) -> (PublicKey, &Message, [u8; 64]) {
    (self.public_key, &self.message, self.signature.to_bytes())
  }

  pub fn message(&self) -> &Message {
    &self.message
  }

  pub(crate) fn new(
    message: Message,
    public_key: PublicKey,
    signature: ed25519_dalek::Signature,
  ) -> Self {
    Self {
      message,
      public_key,
      signature,
    }
  }

  pub fn public_key(&self) -> PublicKey {
    self.public_key
  }

  pub(crate) fn verify(&self, fingerprint: Fingerprint) -> Result {
    ensure! {
      fingerprint == self.message.fingerprint,
      error::SignatureFingerprintMismatch {
        signature: self.message.fingerprint,
        package: fingerprint,
      },
    }

    self
      .public_key
      .inner()
      .verify_strict(self.message.serialize().as_bytes(), &self.signature)
      .map_err(DalekSignatureError)
      .context(error::SignatureInvalid {
        public_key: self.public_key,
      })
  }
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32Encoder::new(Bech32Type::Signature);
    encoder.bytes(&self.public_key.inner().to_bytes());
    encoder.bytes(self.message.fingerprint.as_bytes());
    encoder.bytes(&self.message.time.unwrap_or_default().to_le_bytes());
    encoder.bytes(&self.signature.to_bytes());
    write!(f, "{encoder}")
  }
}

impl FromStr for Signature {
  type Err = SignatureError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut decoder = Bech32Decoder::new(Bech32Type::Signature, s)?;
    let public_key = decoder.byte_array()?;
    let fingerprint = decoder.byte_array()?;
    let time = u128::from_le_bytes(decoder.byte_array()?);
    let signature = decoder.byte_array()?;
    decoder.done()?;
    Ok(Self {
      message: Message {
        fingerprint: Fingerprint::from_bytes(fingerprint),
        time: if time == 0 { None } else { Some(time) },
      },
      signature: ed25519_dalek::Signature::from_bytes(&signature),
      public_key: PublicKey::from_bytes(public_key).context(signature_error::PublicKey)?,
    })
  }
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
  fn signature_begins_with_pubkey() {
    let prefix = format!(
      "signature1a{}{}",
      &test::PUBLIC_KEY["public1a".len()..test::PUBLIC_KEY.len() - 6],
      &test::FINGERPRINT["package1a".len()..test::FINGERPRINT.len() - 6]
    );
    assert!(test::SIGNATURE.starts_with(&prefix));
  }

  #[test]
  fn modifying_fingerprint_invalidates_signature() {
    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();
    let message = Message {
      fingerprint,
      time: Some(1000),
    };
    let mut signature = private_key.sign(&message, &message.serialize());
    signature.message.fingerprint = Fingerprint::from_bytes(default());
    assert_matches!(
      signature.verify(fingerprint).unwrap_err(),
      Error::SignatureFingerprintMismatch { .. },
    );
  }

  #[test]
  fn modifying_time_invalidates_signature() {
    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();
    let message = Message {
      fingerprint,
      time: Some(1000),
    };
    let mut signature = private_key.sign(&message, &message.serialize());
    signature.message.time = Some(2000);
    assert_matches!(
      signature.verify(fingerprint).unwrap_err(),
      Error::SignatureInvalid { .. },
    );
  }

  #[test]
  fn removing_time_invalidates_signature() {
    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();
    let message = Message {
      fingerprint,
      time: Some(1000),
    };
    let mut signature = private_key.sign(&message, &message.serialize());
    signature.message.time = None;
    assert_matches!(
      signature.verify(fingerprint).unwrap_err(),
      Error::SignatureInvalid { .. },
    );
  }
}
