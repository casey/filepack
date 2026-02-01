use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PrivateKey(ed25519_dalek::SigningKey);

impl PrivateKey {
  pub(crate) const LEN: usize = ed25519_dalek::SECRET_KEY_LENGTH;

  pub(crate) fn as_secret_bytes(&self) -> [u8; Self::LEN] {
    self.0.to_bytes()
  }

  pub fn display_secret(&self) -> DisplaySecret {
    DisplaySecret(self.clone())
  }

  pub fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
    let inner = ed25519_dalek::SigningKey::from_bytes(&bytes);
    assert!(!inner.verifying_key().is_weak());
    Self(inner)
  }

  pub(crate) fn generate() -> Self {
    let inner = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let verifying_key = inner.verifying_key();
    assert!(!verifying_key.is_weak());
    Self(inner)
  }

  pub(crate) fn inner_secret(&self) -> &ed25519_dalek::SigningKey {
    &self.0
  }

  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    let private_key = filesystem::read_to_string_opt(path)?
      .ok_or_else(|| error::PrivateKeyNotFound { path }.build())?;

    let private_key = private_key
      .trim()
      .parse::<Self>()
      .context(error::PrivateKeyLoad { path })?;

    Ok(private_key)
  }

  pub fn public_key(&self) -> PublicKey {
    self.clone().into()
  }

  pub(crate) fn sign(&self, message: &Message) -> Signature {
    use ed25519_dalek::Signer;
    Signature::new(
      message.clone(),
      self.public_key(),
      self.0.sign(message.fingerprint().as_bytes()),
    )
  }
}

impl FromStr for PrivateKey {
  type Err = PrivateKeyError;

  fn from_str(key: &str) -> Result<Self, Self::Err> {
    let mut decoder = Bech32Decoder::new(Bech32Type::PrivateKey, key)?;
    let public_key = decoder.byte_array::<{ PublicKey::LEN }>()?;
    let private_key = decoder.byte_array::<{ Self::LEN }>()?;
    decoder.done()?;

    let inner = ed25519_dalek::SigningKey::from_bytes(&private_key);
    assert!(!inner.verifying_key().is_weak());

    ensure!(
      inner.verifying_key().to_bytes() == public_key,
      private_key_error::Mismatch,
    );

    Ok(Self(inner))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let key = PrivateKey::generate();
    assert_eq!(
      key
        .display_secret()
        .to_string()
        .parse::<PrivateKey>()
        .unwrap(),
      key
    );
  }

  #[test]
  fn private_key_begins_with_public_key() {
    let prefix = format!(
      "private1a{}",
      &test::PUBLIC_KEY["public1a".len()..test::PUBLIC_KEY.len() - 6],
    );
    assert!(test::PRIVATE_KEY.starts_with(&prefix));
  }

  #[test]
  fn public_key_mismatch_error() {
    let other = PrivateKey::generate();
    let other_public_key_data =
      &other.public_key().to_string()["public1a".len()..test::PUBLIC_KEY.len() - 6];
    let public_key_data_len = test::PUBLIC_KEY.len() - "public1a".len() - 6;
    let private_key_data =
      &test::PRIVATE_KEY["private1a".len() + public_key_data_len..test::PRIVATE_KEY.len() - 6];
    let mismatched = test::checksum(&format!(
      "private1a{other_public_key_data}{private_key_data}"
    ));
    assert_eq!(
      mismatched.parse::<PrivateKey>().unwrap_err().to_string(),
      "private key public key mismatch",
    );
  }

  #[test]
  fn serialized_private_key_is_not_valid_public_key() {
    test::PRIVATE_KEY.parse::<PublicKey>().unwrap_err();
  }

  #[test]
  fn whitespace_is_not_trimmed_when_parsing_from_string() {
    format!(" {}", test::PRIVATE_KEY)
      .parse::<PrivateKey>()
      .unwrap_err();
  }

  #[test]
  fn whitespace_is_trimmed_when_loading_from_disk() {
    let dir = tempdir();

    filesystem::chmod(Utf8Path::from_path(dir.path()).unwrap(), 0o700).unwrap();

    let path = Utf8PathBuf::from_path_buf(dir.path().join("key")).unwrap();

    filesystem::write(&path, format!(" \t{}\n", test::PRIVATE_KEY)).unwrap();

    filesystem::chmod(&path, 0o600).unwrap();

    assert_eq!(
      PrivateKey::load(&path).unwrap(),
      test::PRIVATE_KEY.parse().unwrap(),
    );
  }
}
