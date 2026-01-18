use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Hash(blake3::Hash);

impl Hash {
  pub(crate) const LEN: usize = blake3::OUT_LEN;

  #[must_use]
  pub fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }

  #[cfg(test)]
  pub(crate) fn bytes(input: &[u8]) -> Self {
    Self(blake3::hash(input))
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
    self.0.fmt(f)
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
