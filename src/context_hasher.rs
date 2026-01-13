use super::*;

pub(crate) struct ContextHasher {
  hasher: Hasher,
  tag: u64,
}

impl ContextHasher {
  pub(crate) fn field(&mut self, tag: u64, field: &[u8]) {
    assert!(tag >= self.tag, "unexpected tag {tag}");
    self.tag = tag;
    self.integer(tag);
    self.integer(field.len().into_u64());
    self.hasher.update(field);
  }

  pub(crate) fn finalize(self) -> Hash {
    self.hasher.finalize().into()
  }

  fn integer(&mut self, n: u64) {
    self.hasher.update(&n.to_le_bytes());
  }

  pub(crate) fn new(context: HashContext) -> Self {
    Self {
      hasher: Hasher::new_derive_key(&format!("filepack:{context}")),
      tag: 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn contexts_produce_distinct_hashes() {
    let mut hashes = HashSet::new();
    for context in HashContext::iter() {
      assert!(hashes.insert(ContextHasher::new(context).finalize()));
    }
  }

  #[test]
  fn field_values_contribute_to_hash() {
    let mut hashes = HashSet::new();
    for value in 0..2 {
      let mut hasher = ContextHasher::new(HashContext::Directory);
      hasher.field(0, &[value]);
      assert!(hashes.insert(hasher.finalize()));
    }
  }

  #[test]
  #[should_panic(expected = "unexpected tag 0")]
  fn tag_order() {
    let mut hasher = ContextHasher::new(HashContext::File);
    hasher.field(1, &[]);
    hasher.field(0, &[]);
  }
}
