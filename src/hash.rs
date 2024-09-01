use super::*;

#[derive(Debug)]
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
