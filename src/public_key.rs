use super::*;

#[derive(Clone, Copy, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct PublicKey(ed25519_dalek::VerifyingKey);

impl PublicKey {
  const LEN: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;

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

  pub fn verify(&self, fingerprint: Hash, signature: &Signature) -> Result<()> {
    let message = Message { fingerprint }.digest();
    self
      .0
      .verify_strict(message.as_bytes(), signature.as_ref())
      .map_err(SignatureError)
      .context(error::SignatureInvalid { public_key: *self })
  }
}

impl From<PrivateKey> for PublicKey {
  fn from(private_key: PrivateKey) -> Self {
    Self(private_key.inner_secret().verifying_key())
  }
}

impl FromStr for PublicKey {
  type Err = PublicKeyError;

  fn from_str(key: &str) -> Result<Self, Self::Err> {
    let bytes = hex::decode(key).context(public_key_error::Hex { key })?;

    if !is_lowercase_hex(key) {
      return Err(public_key_error::Case { key }.build());
    }

    let array: [u8; Self::LEN] = bytes
      .as_slice()
      .try_into()
      .context(public_key_error::Length {
        key,
        length: bytes.len(),
      })?;

    let inner = ed25519_dalek::VerifyingKey::from_bytes(&array)
      .map_err(SignatureError)
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
    write!(f, "{}", hex::encode(self.0.to_bytes()))
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
  fn must_have_leading_zeros() {
    "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<PublicKey>()
      .unwrap();

    "f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<PublicKey>()
      .unwrap_err();
  }

  #[test]
  fn parse() {
    let key = PrivateKey::generate().public_key();
    assert_eq!(key.to_string().parse::<PublicKey>().unwrap(), key);
  }

  #[test]
  fn parse_hex_error() {
    assert_eq!(
      "xyz".parse::<PublicKey>().unwrap_err().to_string(),
      "invalid public key hex: `xyz`"
    );
  }

  #[test]
  fn parse_length_error() {
    assert_eq!(
      "0123".parse::<PublicKey>().unwrap_err().to_string(),
      "invalid public key byte length 2: `0123`"
    );
  }

  #[test]
  fn uppercase_is_forbidden() {
    let key = "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3";
    key.parse::<PublicKey>().unwrap();
    assert_eq!(
      key
        .to_uppercase()
        .parse::<PublicKey>()
        .unwrap_err()
        .to_string(),
      "public keys must be lowercase hex: \
      `0F6D444F09EB336D3CC94D66CC541FEA0B70B36BE291EB3ECF5B49113F34C8D3`"
    );
  }

  #[test]
  fn weak_public_keys_are_forbidden() {
    assert!(matches!(
        "0000000000000000000000000000000000000000000000000000000000000000"
          .parse::<PublicKey>()
          .unwrap_err(),
        PublicKeyError::Weak { key }
          if key == "0000000000000000000000000000000000000000000000000000000000000000",
    ));
  }

  #[test]
  fn whitespace_is_not_trimmed_when_parsing_from_string() {
    "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<PublicKey>()
      .unwrap();

    " 0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<PublicKey>()
      .unwrap_err();
  }

  #[test]
  fn whitespace_is_trimmed_when_loading_from_disk() {
    let dir = tempdir();

    let path = Utf8PathBuf::from_path_buf(dir.path().join("key")).unwrap();

    let key = "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<PublicKey>()
      .unwrap();

    filesystem::write(&path, format!(" \t{key}\n")).unwrap();

    assert_eq!(PublicKey::load(&path).unwrap(), key);
  }
}
