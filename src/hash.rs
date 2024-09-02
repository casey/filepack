use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hash(blake3::Hash);

impl Hash {
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
    let output = serde_json::from_str::<Hash>(&json).unwrap();
    assert_eq!(output, input);
  }
}
