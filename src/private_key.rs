use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PrivateKey(ed25519_dalek::SigningKey);

impl PrivateKey {
  pub(crate) const LEN: usize = ed25519_dalek::SECRET_KEY_LENGTH;

  pub(crate) fn as_secret_bytes(&self) -> [u8; Self::LEN] {
    self.0.to_bytes()
  }

  pub(crate) fn display_private_key(&self) -> DisplayPrivateKey {
    DisplayPrivateKey::new(self)
  }

  pub(crate) fn generate() -> Self {
    let inner = ed25519_dalek::SigningKey::generate(&mut rand::rng());
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

  pub(crate) fn sign<T: Message>(&self, message: T) -> Signature<T> {
    use ed25519_dalek::Signer;
    let signature = self.0.sign(message.digest(Version::Zero).as_bytes());
    Signature::new(self.public_key(), message, signature)
  }
}

impl FromStr for PrivateKey {
  type Err = PrivateKeyError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(s.parse::<DisplayPrivateKey>()?.inner()))
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
        .display_private_key()
        .to_string()
        .parse::<PrivateKey>()
        .unwrap(),
      key
    );
  }

  #[test]
  fn private_key_begins_with_public_key() {
    let prefix = format!("private1{}", &test::PUBLIC_KEY["public1".len()..]);
    assert!(test::PRIVATE_KEY.starts_with(&prefix));
  }

  #[test]
  fn public_key_mismatch_error() {
    let other = PrivateKey::generate().public_key().to_string();
    let mismatched = format!(
      "private1{}{}",
      &other["public1".len()..],
      &test::PRIVATE_KEY["private1".len() + 64..],
    );
    assert_eq!(
      mismatched.parse::<PrivateKey>().unwrap_err().to_string(),
      "private key derived public key does not match embedded public key",
    );
  }

  #[test]
  fn serialized_private_key_is_not_valid_public_key() {
    assert_eq!(
      test::PRIVATE_KEY
        .parse::<PublicKey>()
        .unwrap_err()
        .to_string(),
      "expected public key with tag `public1…` but found `private1…`",
    );
  }

  #[test]
  fn whitespace_is_not_trimmed_when_parsing_from_string() {
    format!(" {}", test::PRIVATE_KEY)
      .parse::<PrivateKey>()
      .unwrap_err();
  }

  #[test]
  fn whitespace_is_trimmed_when_loading_from_disk() {
    let (_dir, path) = tempdir();

    filesystem::chmod(&path, 0o700).unwrap();

    let path = path.join("key");

    filesystem::write(&path, format!(" \t{}\n", test::PRIVATE_KEY)).unwrap();

    filesystem::chmod(&path, 0o600).unwrap();

    assert_eq!(
      PrivateKey::load(&path).unwrap(),
      test::PRIVATE_KEY.parse().unwrap(),
    );
  }
}
