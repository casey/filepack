use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Debug, Decode, Encode, EnumDiscriminants, PartialEq)]
#[strum_discriminants(
  allow(clippy::arbitrary_source_item_ordering),
  derive(Display),
  name(EntryType),
  strum(serialize_all = "kebab-case")
)]
pub enum Entry {
  #[n(0)]
  File {
    #[n(0)]
    hash: Hash,
    #[n(1)]
    size: u64,
  },
  #[n(1)]
  Directory {
    #[n(0)]
    hash: Hash,
    #[n(1)]
    size: u64,
    #[n(2)]
    totals: Totals,
  },
}

impl Entry {
  pub fn file(hash: Hash, size: u64) -> Self {
    Self::File { hash, size }
  }

  pub fn directory(hash: Hash, size: u64, totals: Totals) -> Self {
    Self::Directory { hash, size, totals }
  }

  pub fn formatted_size(&self) -> SizeFormatter<u64, FormatSizeOptions> {
    format_size(self.size())
  }

  pub fn hash(&self) -> Hash {
    match self {
      Self::File { hash, .. } | Self::Directory { hash, .. } => *hash,
    }
  }

  pub fn size(&self) -> u64 {
    match self {
      Self::File { size, .. } | Self::Directory { size, .. } => *size,
    }
  }

  pub fn ty(&self) -> EntryType {
    self.discriminant()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Entry::File {
      size: 100,
      hash: Hash::bytes(b"foo"),
    });

    assert_encoding(Entry::Directory {
      hash: Hash::bytes(b"foo"),
      size: 100,
      total_file_size: 200,
    });
  }
}
