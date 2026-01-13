use super::*;

pub(crate) struct FingerprintHasher {
  hasher: Hasher,
  tag: u64,
}

impl FingerprintHasher {
  pub(crate) fn field(&mut self, tag: u64, field: &[u8]) {
    assert!(tag >= self.tag, "unexpected tag {tag}");
    self.tag = tag;
    self.hasher.update(&tag.to_le_bytes());
    self.hasher.update(&field.len().into_u64().to_le_bytes());
    self.hasher.update(field);
  }

  pub(crate) fn finalize(self) -> Hash {
    self.hasher.finalize().into()
  }

  pub(crate) fn new(context: FingerprintPrefix) -> Self {
    let mut hasher = Hasher::new();
    let prefix = context.prefix();
    hasher.update(&prefix.len().into_u64().to_le_bytes());
    hasher.update(prefix.as_bytes());
    Self { hasher, tag: 0 }
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
