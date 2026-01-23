use super::*;

pub(crate) struct DisplaySecret(pub(crate) PrivateKey);

impl Display for DisplaySecret {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32mEncoder::new(Bech32mType::PrivateKey);
    encoder.bytes(&self.0.as_secret_bytes());
    write!(f, "{encoder}")
  }
}
