use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub enum Error {
  #[snafu(display("invalid private key hex"))]
  Hex { source: hex::FromHexError },
  #[snafu(display("invalid private key byte length {length}"))]
  Length { length: usize },
  #[snafu(display("weak private key"))]
  Weak,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrivateKey(ed25519_dalek::SigningKey);

impl PrivateKey {
  const LEN: usize = ed25519_dalek::SECRET_KEY_LENGTH;

  pub(crate) fn as_bytes(&self) -> [u8; Self::LEN] {
    self.0.to_bytes()
  }

  pub(crate) fn display_secret(&self) -> DisplaySecret {
    DisplaySecret(self.clone())
  }

  pub(crate) fn generate() -> Self {
    let inner = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let verifying_key = inner.verifying_key();
    assert!(!verifying_key.is_weak());
    Self(inner)
  }

  pub(crate) fn inner(&self) -> &ed25519_dalek::SigningKey {
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

  pub(crate) fn load_and_sign(path: &Utf8Path, message: Hash) -> Result<(PublicKey, Signature)> {
    let private_key = Self::load(path)?;

    let signature = private_key.sign(message);

    Ok((private_key.public_key(), signature))
  }

  #[must_use]
  pub fn public_key(&self) -> PublicKey {
    self.clone().into()
  }

  pub(crate) fn sign(&self, fingerprint: Hash) -> Signature {
    use ed25519_dalek::Signer;
    let message = Message { fingerprint }.digest();
    self.0.sign(message.as_bytes()).into()
  }
}

impl FromStr for PrivateKey {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let bytes = hex::decode(s).context(HexError)?;

    let secret: [u8; Self::LEN] = bytes.as_slice().try_into().ok().context(LengthError {
      length: bytes.len(),
    })?;

    let inner = ed25519_dalek::SigningKey::from_bytes(&secret);

    ensure! {
      !inner.verifying_key().is_weak(),
      WeakError,
    }

    Ok(Self(inner))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn must_have_leading_zeros() {
    "0e56ae8b43aa93fd4c179ceaff96f729522622d26b4b5357bc959e476e59e107"
      .parse::<PrivateKey>()
      .unwrap();

    "e56ae8b43aa93fd4c179ceaff96f729522622d26b4b5357bc959e476e59e107"
      .parse::<PrivateKey>()
      .unwrap_err();
  }

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
  fn parse_hex_error() {
    assert_eq!(
      "xyz".parse::<PrivateKey>().unwrap_err().to_string(),
      "invalid private key hex"
    );
  }

  #[test]
  fn parse_length_error() {
    assert_eq!(
      "0123".parse::<PrivateKey>().unwrap_err().to_string(),
      "invalid private key byte length 2"
    );
  }

  #[test]
  fn whitespace_is_not_trimmed_when_parsing_from_string() {
    "0e56ae8b43aa93fd4c179ceaff96f729522622d26b4b5357bc959e476e59e107"
      .parse::<PrivateKey>()
      .unwrap();

    " 0e56ae8b43aa93fd4c179ceaff96f729522622d26b4b5357bc959e476e59e107"
      .parse::<PrivateKey>()
      .unwrap_err();
  }

  #[test]
  fn whitespace_is_trimmed_when_loading_from_disk() {
    let dir = tempdir();

    filesystem::chmod(Utf8Path::from_path(dir.path()).unwrap(), 0o700).unwrap();

    let path = Utf8PathBuf::from_path_buf(dir.path().join("key")).unwrap();

    let key = "0e56ae8b43aa93fd4c179ceaff96f729522622d26b4b5357bc959e476e59e107"
      .parse::<PrivateKey>()
      .unwrap();

    filesystem::write(&path, format!(" \t{}\n", key.display_secret())).unwrap();

    filesystem::chmod(&path, 0o600).unwrap();

    assert_eq!(PrivateKey::load(&path).unwrap(), key);
  }
}
