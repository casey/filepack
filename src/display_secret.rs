use super::*;

pub struct DisplaySecret(pub(crate) PrivateKey);

impl Display for DisplaySecret {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32Encoder::new(Bech32Type::PrivateKey);
    encoder.bytes(&self.0.public_key().inner().to_bytes());
    encoder.bytes(&self.0.as_secret_bytes());
    write!(f, "{encoder}")
  }
}
