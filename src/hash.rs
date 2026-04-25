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
      &[
        0x58, 0x20, 0x04, 0xe0, 0xbb, 0x39, 0xf3, 0x0b, 0x1a, 0x3f, 0xeb, 0x89, 0xf5, 0x36, 0xc9,
        0x3b, 0xe1, 0x50, 0x55, 0x48, 0x2d, 0xf7, 0x48, 0x67, 0x4b, 0x00, 0xd2, 0x6e, 0x5a, 0x75,
        0x77, 0x77, 0x02, 0xe9,
      ],
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
