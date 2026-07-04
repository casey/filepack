use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct Entry {
  #[n(0)]
  pub ty: EntryType,
  #[n(1)]
  pub hash: Hash,
  #[n(2)]
  pub size: u64,
  #[n(3)]
  pub total_file_size: Option<u64>,
}

impl Entry {
  pub fn formatted_size(&self) -> SizeFormatter<u64, FormatSizeOptions> {
    SizeFormatter::new(self.size, FormatSizeOptions::from(BINARY).decimal_places(1))
  }
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
      total_file_size: None,
    });
  }
}
