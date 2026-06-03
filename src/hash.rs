use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Hash(blake3::Hash);

impl Hash {
  pub(crate) const LEN: usize = blake3::OUT_LEN;

  pub fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }

  pub fn bytes(input: &[u8]) -> Self {
    Self(blake3::hash(input))
  }
}

impl Decode for Hash {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    Ok(Self::from(decoder.byte_array()?))
  }
}

impl From<blake3::Hash> for Hash {
  fn from(hash: blake3::Hash) -> Self {
    Self(hash)
  }
}

impl From<Fingerprint> for Hash {
  fn from(fingerprint: Fingerprint) -> Self {
    fingerprint.0
  }
}

impl From<[u8; Hash::LEN]> for Hash {
  fn from(bytes: [u8; Hash::LEN]) -> Self {
    Self(bytes.into())
  }
}

impl FromStr for Hash {
  type Err = HashError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let hash = s.parse()?;

    if !is_lowercase_hex(s) {
      return Err(hash_error::Case { hash: s }.build());
    }

    Ok(Self(hash))
  }
}

impl Ord for Hash {
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_bytes().cmp(other.as_bytes())
  }
}

impl PartialOrd for Hash {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Serialize for Hash {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.0.to_string().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Hash {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    use serde::de::{Error, Unexpected};

    let s = String::deserialize(deserializer)?;

    Ok(Self(s.parse::<blake3::Hash>().map_err(|_| {
      D::Error::invalid_value(Unexpected::Str(&s), &"64 hex digits")
    })?))
  }
}

impl Display for Hash {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Display::fmt(&self.0, f)
  }
}

impl Encode for Hash {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_bytes().encode(encoder);
  }
}

impl redb::Key for Hash {
  fn compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)
  }
}

impl redb::Value for Hash {
  type AsBytes<'a>
    = &'a [u8; Self::LEN]
  where
    Self: 'a;

  type SelfType<'a>
    = Hash
  where
    Self: 'a;

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    value.as_bytes()
  }

  fn fixed_width() -> Option<usize> {
    Some(Self::LEN)
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    <[u8; Self::LEN]>::try_from(data).unwrap().into()
  }

  fn type_name() -> redb::TypeName {
    TypeName::Hash.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn deserialize_error_format() {
    assert_eq!(
      serde_json::from_str::<Hash>("\"foo\"")
        .unwrap_err()
        .to_string(),
      r#"invalid value: string "foo", expected 64 hex digits"#,
    );
  }

  #[test]
  fn hash() {
    assert_cbor(
      Hash::bytes(b"foo"),
      "582004e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9",
    );
  }

  #[test]
  fn serde() {
    let input = Hash::bytes(&[]);
    let json = serde_json::to_string(&input).unwrap();
    assert_eq!(
      json,
      "\"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262\""
    );
    assert_eq!(serde_json::from_str::<Hash>(&json).unwrap(), input);
  }

  #[test]
  fn uppercase_is_forbidden() {
    test::HASH.to_uppercase().parse::<Hash>().unwrap_err();
  }
}
