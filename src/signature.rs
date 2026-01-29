use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Signature {
  fingerprint: Fingerprint,
  public_key: PublicKey,
  signature: ed25519_dalek::Signature,
}

impl Signature {
  fn comparison_key(&self) -> (PublicKey, Fingerprint, [u8; 64]) {
    (self.public_key, self.fingerprint, self.signature.to_bytes())
  }

  pub(crate) fn new(
    fingerprint: Fingerprint,
    public_key: PublicKey,
    signature: ed25519_dalek::Signature,
  ) -> Self {
    Self {
      fingerprint,
      signature,
      public_key,
    }
  }

  pub(crate) fn public_key(&self) -> PublicKey {
    self.public_key
  }

  pub(crate) fn verify(&self, fingerprint: Fingerprint, message: &SerializedMessage) -> Result {
    ensure! {
      fingerprint == self.fingerprint,
      error::SignatureFingerprintMismatch,
    }

    self
      .public_key
      .inner()
      .verify_strict(message.as_bytes(), &self.signature)
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
    encoder.bytes(self.fingerprint.as_bytes());
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
    let signature = decoder.byte_array()?;
    decoder.done()?;
    Ok(Self {
      fingerprint: Fingerprint::from_bytes(fingerprint),
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
}
