use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq)]
pub struct Entry {
  #[n(1)]
  pub(crate) hash: Hash,
  #[n(2)]
  pub(crate) size: u64,
  #[n(3)]
  pub(crate) info: EntryInfo,
}

impl Entry {
  pub fn directory(hash: Hash, size: u64, totals: Totals) -> Self {
    Self {
      hash,
      size,
      info: EntryInfo::Directory { totals },
    }
  }

  pub fn file(hash: Hash, size: u64) -> Self {
    Self {
      hash,
      size,
      info: EntryInfo::File,
    }
  }

  pub(crate) fn ty(&self) -> EntryType {
    self.info.discriminant()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn directory_encoding() {
    assert_encoding(Entry::directory(
      Hash::bytes(b"foo"),
      1,
      Totals {
        directories: 2,
        directory_size: 3,
        file_size: 4,
        files: 5,
      },
    ));
  }

  #[test]
  fn file_encoding() {
    assert_encoding(Entry::file(Hash::bytes(b"foo"), 1));
  }
}
