use super::*;

pub(crate) struct FingerprintHasher {
  hasher: Hasher,
  tag: u64,
}

impl FingerprintHasher {
  pub(crate) fn field(&mut self, tag: u64, field: &[u8]) {
    assert!(tag >= self.tag, "unexpected tag {tag}");
    self.tag = tag;
    self.varint(tag);
    self.varint(field.len().into_u64());
    self.hasher.update(field);
  }

  pub(crate) fn finalize(self) -> Hash {
    self.hasher.finalize().into()
  }

  pub(crate) fn new(context: FingerprintPrefix) -> Self {
    let mut hasher = Self {
      hasher: Hasher::new(),
      tag: 0,
    };
    let prefix = context.prefix();
    hasher.varint(prefix.len().into_u64());
    hasher.hasher.update(prefix.as_bytes());
    hasher
  }

  fn varint(&mut self, mut n: u64) {
    loop {
      let byte = (n & 0b0111_1111).try_into().unwrap();
      n >>= 7;
      if n == 0 {
        self.hasher.update(&[byte]);
        break;
      }
      self.hasher.update(&[byte | 0b1000_0000]);
    }
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
