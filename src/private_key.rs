use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PrivateKey(ed25519_dalek::SigningKey);

impl PrivateKey {
  pub(crate) const LEN: usize = ed25519_dalek::SECRET_KEY_LENGTH;

  pub(crate) fn as_secret_bytes(&self) -> [u8; Self::LEN] {
    self.0.to_bytes()
  }

  pub(crate) fn display_secret(&self) -> DisplaySecret {
    DisplaySecret(self.clone())
  }

  #[cfg(test)]
  pub(crate) fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
    Self(ed25519_dalek::SigningKey::from_bytes(&bytes))
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

  #[must_use]
  pub fn public_key(&self) -> PublicKey {
    self.clone().into()
  }

  pub(crate) fn sign(&self, message: &SerializedMessage) -> Signature {
    use ed25519_dalek::Signer;
    Signature::new(SignatureScheme::Filepack, self.0.sign(message.as_bytes()))
  }
}

impl Bech32m<0, { PrivateKey::LEN }> for PrivateKey {
  const TYPE: Bech32mType = Bech32mType::PrivateKey;
  type Suffix = ();
}

impl FromStr for PrivateKey {
  type Err = Bech32mError;

  fn from_str(key: &str) -> Result<Self, Self::Err> {
    let mut decoder = Bech32mDecoder::new(Bech32mType::PrivateKey, key)?;
    let inner = decoder.byte_array()?;
    decoder.done()?;

    let inner = ed25519_dalek::SigningKey::from_bytes(&inner);
    assert!(!inner.verifying_key().is_weak());
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
