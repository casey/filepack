use super::*;

pub(crate) struct FingerprintHasher(FingerprintSerializer<Hasher>);

impl FingerprintHasher {
  pub(crate) fn field(&mut self, tag: u64, field: &[u8]) {
    self.0.field(tag, field).unwrap();
  }

  pub(crate) fn finalize(self) -> Hash {
    self.0.into_inner().finalize().into()
  }

  pub(crate) fn new(context: FingerprintPrefix) -> Self {
    Self(FingerprintSerializer::new(context, Hasher::new()).unwrap())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn contexts_produce_distinct_hashes() {
    let mut hashes = HashSet::new();
    for context in FingerprintPrefix::iter() {
      assert!(hashes.insert(FingerprintHasher::new(context).finalize()));
    }
  }

  #[test]
  fn field_values_contribute_to_hash() {
    let mut hashes = HashSet::new();
    for value in 0..2 {
      let mut hasher = FingerprintHasher::new(FingerprintPrefix::Directory);
      hasher.field(0, &[value]);
      assert!(hashes.insert(hasher.finalize()));
    }
  }

  #[test]
  #[should_panic(expected = "unexpected tag 0")]
  fn tag_order() {
    let mut hasher = FingerprintHasher::new(FingerprintPrefix::File);
    hasher.field(1, &[]);
    hasher.field(0, &[]);
  }
}
