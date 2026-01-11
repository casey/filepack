use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub enum Error {
  #[snafu(display("invalid public key hex: `{key}`"))]
  Hex {
    key: String,
    source: hex::FromHexError,
  },
  #[snafu(display("invalid public key: `{key}`"))]
  Key { key: String, source: SignatureError },
  #[snafu(display("invalid public key byte length {length}: `{key}`"))]
  Length {
    key: String,
    length: usize,
    source: TryFromSliceError,
  },
  #[snafu(display("weak public key: `{key}`"))]
  Weak { key: String },
}

#[derive(Clone, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct PublicKey(ed25519_dalek::VerifyingKey);

impl PublicKey {
  const LEN: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;

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
      .context(error::SignatureInvalid {
        public_key: self.clone(),
      })
  }
}

impl From<PrivateKey> for PublicKey {
  fn from(private_key: PrivateKey) -> Self {
    Self(private_key.inner().verifying_key())
  }
}

impl FromStr for PublicKey {
  type Err = Error;

  fn from_str(key: &str) -> Result<Self, Self::Err> {
    let bytes = hex::decode(key).context(HexError { key })?;

    let array: [u8; Self::LEN] = bytes.as_slice().try_into().context(LengthError {
      key,
      length: bytes.len(),
    })?;

    let inner = ed25519_dalek::VerifyingKey::from_bytes(&array)
      .map_err(SignatureError)
      .context(KeyError { key })?;

    ensure! {
      !inner.is_weak(),
      WeakError { key },
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
  fn weak_public_keys_are_forbidden() {
    assert!(matches!(
        "0000000000000000000000000000000000000000000000000000000000000000"
          .parse::<PublicKey>()
          .unwrap_err(),
        Error::Weak { key }
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
    let dir = TempDir::new().unwrap();

    let path = Utf8PathBuf::from_path_buf(dir.join("key")).unwrap();

    let key = "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<PublicKey>()
      .unwrap();

    filesystem::write(&path, format!(" \t{key}\n")).unwrap();

    assert_eq!(PublicKey::load(&path).unwrap(), key);
  }
}
