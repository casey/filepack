use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hash(blake3::Hash);

impl Hash {
  const LEN: usize = blake3::OUT_LEN;

  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }

  pub(crate) fn bytes(input: &[u8]) -> Self {
    Self(blake3::hash(input))
  }
}

impl From<blake3::Hash> for Hash {
  fn from(hash: blake3::Hash) -> Self {
    Self(hash)
  }
}

impl From<Hash> for [u8; Hash::LEN] {
  fn from(hash: Hash) -> Self {
    hash.0.into()
  }
}

impl FromStr for Hash {
  type Err = blake3::HexError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(s.parse()?))
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
    self.0.fmt(f)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
  fn deserialize_error_format() {
    assert_eq!(
      serde_json::from_str::<Hash>("\"foo\"")
        .unwrap_err()
        .to_string(),
      r#"invalid value: string "foo", expected 64 hex digits"#,
    );
  }
}
