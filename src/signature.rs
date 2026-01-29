use super::*;

#[derive(Clone, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Signature {
  fingerprint: Fingerprint,
  inner: ed25519_dalek::Signature,
  public_key: PublicKey,
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

impl Signature {
  fn comparison_key(&self) -> (PublicKey, Fingerprint, [u8; 64]) {
    (self.public_key, self.fingerprint, self.inner.to_bytes())
  }

  pub(crate) fn new(
    fingerprint: Fingerprint,
    inner: ed25519_dalek::Signature,
    public_key: PublicKey,
  ) -> Self {
    Self {
      fingerprint,
      inner,
      public_key,
    }
  }

  pub(crate) fn public_key(&self) -> PublicKey {
    self.public_key
  }

  pub(crate) fn verify(&self, message: &SerializedMessage) -> Result {
    self
      .public_key
      .inner()
      .verify_strict(message.as_bytes(), &self.inner)
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
    encoder.bytes(&self.inner.to_bytes());
    write!(f, "{encoder}")
  }
}

impl fmt::Debug for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.inner, f)
  }
}

impl FromStr for Signature {
  type Err = SignatureError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut decoder = Bech32Decoder::new(Bech32Type::Signature, s)?;
    let public_key = decoder.byte_array()?;
    let fingerprint = decoder.byte_array()?;
    let inner = decoder.byte_array()?;
    decoder.done()?;
    Ok(Self {
      fingerprint: Fingerprint::from_bytes(fingerprint),
      inner: ed25519_dalek::Signature::from_bytes(&inner),
      public_key: PublicKey::from_bytes(public_key).context(signature_error::PublicKey)?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn signature_begins_with_pubkey() {
    assert!(test::SIGNATURE.starts_with(
      &test::PUBLIC_KEY[..test::PUBLIC_KEY.len() - 6].replace("public1", "signature1")
    ),);
  }
}
