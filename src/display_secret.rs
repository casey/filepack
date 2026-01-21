use super::*;

pub(crate) struct DisplaySecret(pub(crate) PrivateKey);

impl Display for DisplaySecret {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    PrivateKey::encode_bech32m(f, Bech32mPayload::from_data(self.0.as_secret_bytes()))
  }
}
