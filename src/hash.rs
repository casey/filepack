use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hash(blake3::Hash);

impl From<blake3::Hash> for Hash {
  fn from(hash: blake3::Hash) -> Self {
    Self(hash)
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

    Ok(Self(s.parse::<blake3::Hash>().map_err(|err| {
      D::Error::invalid_value(Unexpected::Str(&s), &err.to_string().as_str())
    })?))
  }
}

impl Display for Hash {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.0.fmt(f)
  }
}
