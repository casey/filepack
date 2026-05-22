use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Encode, Decode, PartialEq)]
pub struct Entry {
  #[n(0)]
  pub ty: EntryType,
  #[n(1)]
  pub hash: Hash,
  #[n(2)]
  pub size: u64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Entry {
      ty: EntryType::File,
      size: 100,
      hash: Hash::bytes(b"foo"),
    });
  }
}
