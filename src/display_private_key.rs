use super::*;

#[derive(Decode, Encode)]
#[deco(transparent)]
pub(crate) struct DisplayPrivateKey([[u8; 32]; 2]);

impl DisplayPrivateKey {
  pub(crate) fn inner(&self) -> ed25519_dalek::SigningKey {
    ed25519_dalek::SigningKey::from_bytes(&self.0[1])
  }

  pub(crate) fn new(private_key: &PrivateKey) -> Self {
    Self([
      private_key.public_key().inner().to_bytes(),
      private_key.as_secret_bytes(),
    ])
  }
}

impl Hex for DisplayPrivateKey {
  const TAG: Tag = Tag::PrivateKey;
}

impl Display for DisplayPrivateKey {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.format(f)
  }
}

impl FromStr for DisplayPrivateKey {
  type Err = PrivateKeyError;

  fn from_str(key: &str) -> Result<Self, Self::Err> {
    let display_private_key = Self::parse(key)?;

    let private_key = display_private_key.inner();
    let public_key = private_key.verifying_key();
    assert!(!public_key.is_weak());

    ensure!(
      public_key.to_bytes() == display_private_key.0[0],
      private_key_error::PublicKeyMismatch,
    );

    Ok(display_private_key)
  }
}
