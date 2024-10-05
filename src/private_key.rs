use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum Error {
  #[snafu(display("invalid hex"))]
  Hex { source: hex::FromHexError },
  #[snafu(display("invalid length {length}"))]
  Length { length: usize },
  #[snafu(display("weak key"))]
  Weak,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrivateKey(ed25519_dalek::SigningKey);

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
    let private_key = match fs::read_to_string(path) {
      Err(err) if err.kind() == io::ErrorKind::NotFound => {
        return Err(error::PrivateKeyNotFound { path }.build())
      }
      result => result.context(error::Io { path })?,
    };

    let private_key = private_key
      .trim()
      .parse::<Self>()
      .context(error::PrivateKeyLoad { path })?;

    Ok(private_key)
  }

  pub(crate) fn public_key(&self) -> PublicKey {
    self.clone().into()
  }

  pub(crate) fn sign(&self, message: &[u8]) -> Signature {
    use ed25519_dalek::Signer;
    self.0.sign(message).into()
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
  fn must_have_leading_zeros() {
    "0e56ae8b43aa93fd4c179ceaff96f729522622d26b4b5357bc959e476e59e107"
      .parse::<PrivateKey>()
      .unwrap();

    "e56ae8b43aa93fd4c179ceaff96f729522622d26b4b5357bc959e476e59e107"
      .parse::<PrivateKey>()
      .unwrap_err();
  }
}
