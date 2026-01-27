use super::*;

#[derive(Clone, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub struct Signature {
  inner: ed25519_dalek::Signature,
}

impl Signature {
  pub fn new(inner: ed25519_dalek::Signature) -> Self {
    Self { inner }
  }

  pub fn verify(&self, message: &SerializedMessage, public_key: PublicKey) -> Result {
    public_key
      .inner()
      .verify_strict(message.as_bytes(), &self.inner)
      .map_err(DalekSignatureError)
      .context(error::SignatureInvalid { public_key })
  }
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32Encoder::new(Bech32Type::Signature);
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
  type Err = Bech32Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut decoder = Bech32Decoder::new(Bech32Type::Signature, s)?;
    let inner = decoder.byte_array()?;
    Ok(Self {
      inner: ed25519_dalek::Signature::from_bytes(&inner),
    })
  }
}
