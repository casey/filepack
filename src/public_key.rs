use super::*;

#[derive(Clone, Copy, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct PublicKey(ed25519_dalek::VerifyingKey);

impl PublicKey {
  pub(crate) const LEN: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;

  #[cfg(test)]
  pub(crate) fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
    Self(ed25519_dalek::VerifyingKey::from_bytes(&bytes).unwrap())
  }

  #[must_use]
  pub fn inner(&self) -> ed25519_dalek::VerifyingKey {
    self.0
  }

  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    let public_key = filesystem::read_to_string_opt(path)?
      .ok_or_else(|| error::PublicKeyNotFound { path }.build())?;

    let public_key = public_key
      .trim()
      .parse::<Self>()
      .context(error::PublicKeyLoad { path })?;

    Ok(public_key)
  }

  pub fn verify(self, message: &SerializedMessage, signature: &Signature) -> Result {
    signature.verify(message, self)
  }
}

impl Bech32m<0, { PublicKey::LEN }> for PublicKey {
  const HRP: Hrp = Hrp::parse_unchecked("public");
  const TYPE: &'static str = "public key";
  type Suffix = ();
}

impl From<PrivateKey> for PublicKey {
  fn from(private_key: PrivateKey) -> Self {
    Self(private_key.inner_secret().verifying_key())
  }
}

impl FromStr for PublicKey {
  type Err = PublicKeyError;

  fn from_str(key: &str) -> Result<Self, Self::Err> {
    let ([], data, ()) = Self::decode_bech32m(key)?;

    let inner = ed25519_dalek::VerifyingKey::from_bytes(&data)
      .map_err(DalekSignatureError)
      .context(public_key_error::Invalid { key })?;

    ensure! {
      !inner.is_weak(),
      public_key_error::Weak { key },
    }

    Ok(Self(inner))
  }
}

impl Display for PublicKey {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Self::encode_bech32m(f, [], *self.0.as_bytes())
  }
}

impl Ord for PublicKey {
  fn cmp(&self, other: &Self) -> Ordering {
    self.0.as_bytes().cmp(other.0.as_bytes())
  }
}

impl PartialOrd for PublicKey {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let key = PrivateKey::generate().public_key();
    assert_eq!(key.to_string().parse::<PublicKey>().unwrap(), key);
  }

  #[test]
  fn weak_public_keys_are_forbidden() {
    assert_matches!(
      test::WEAK_PUBLIC_KEY.parse::<PublicKey>().unwrap_err(),
      PublicKeyError::Weak { .. },
    );
  }

  #[test]
  fn whitespace_is_not_trimmed_when_parsing_from_string() {
    format!(" {}", test::PUBLIC_KEY)
      .parse::<PublicKey>()
      .unwrap_err();
  }

  #[test]
  fn whitespace_is_trimmed_when_loading_from_disk() {
    let dir = tempdir();

    let path = Utf8PathBuf::from_path_buf(dir.path().join("key")).unwrap();

    filesystem::write(&path, format!(" \t{}\n", test::PUBLIC_KEY)).unwrap();

    assert_eq!(
      PublicKey::load(&path).unwrap().to_string(),
      test::PUBLIC_KEY,
    );
  }
}
