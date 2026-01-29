use super::*;

#[derive(Clone, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Signature {
  inner: ed25519_dalek::Signature,
  public_key: PublicKey,
}

impl Ord for Signature {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .inner
      .to_bytes()
      .cmp(&other.inner.to_bytes())
      .then_with(|| self.public_key.cmp(&other.public_key))
  }
}

impl PartialOrd for Signature {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Signature {
  pub(crate) fn new(inner: ed25519_dalek::Signature, public_key: PublicKey) -> Self {
    Self { inner, public_key }
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
    encoder.bytes(&self.inner.to_bytes());
    encoder.bytes(&self.public_key.inner().to_bytes());
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
    let inner = decoder.byte_array()?;
    let public_key = decoder.byte_array()?;
    decoder.done()?;
    Ok(Self {
      inner: ed25519_dalek::Signature::from_bytes(&inner),
      public_key: PublicKey::from_bytes(public_key).context(signature_error::PublicKey)?,
    })
  }
}
