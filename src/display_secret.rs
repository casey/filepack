use super::*;

pub(crate) struct DisplaySecret(pub(crate) PrivateKey);

impl Display for DisplaySecret {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", hex::encode(self.0.as_bytes()))
  }
}
