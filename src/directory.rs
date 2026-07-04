use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Debug, Default, Encode, Decode, PartialEq)]
pub struct Directory {
  #[n(0)]
  pub version: Version,
  #[n(1)]
  pub entries: BTreeMap<ComponentBuf, Entry>,
}

#[cfg(test)]
impl Directory {
  pub(crate) fn cbor(&self) -> (Vec<u8>, Hash) {
    let cbor = self.encode_to_vec();
    let hash = Hash::bytes(&cbor);
    (cbor, hash)
  }

  pub(crate) fn entry(&self) -> Entry {
    let cbor = self.encode_to_vec();

    Entry {
      ty: EntryType::Directory,
      hash: Hash::bytes(&cbor),
      size: cbor.len().into_u64(),
      total_file_size: Some(
        self
          .entries
          .values()
          .map(|entry| match entry.ty {
            EntryType::File => entry.size,
            EntryType::Directory => entry.total_file_size.unwrap(),
          })
          .sum(),
      ),
    }
  }

  pub(crate) fn insert_directory(&mut self, name: &str, directory: &Directory) {
    self.insert_entry(name, directory.entry());
  }

  fn insert_entry(&mut self, name: &str, entry: Entry) {
    assert!(self.entries.insert(name.parse().unwrap(), entry).is_none());
  }

  pub(crate) fn insert_file(&mut self, name: &str, contents: &[u8]) {
    self.insert_entry(
      name,
      Entry {
        ty: EntryType::File,
        hash: Hash::bytes(contents),
        size: contents.len().into_u64(),
        total_file_size: None,
      },
    );
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Directory {
      version: Version::Zero,
      entries: BTreeMap::from([(
        "foo".parse::<ComponentBuf>().unwrap(),
        Entry {
          ty: EntryType::File,
          size: 0,
          hash: Hash::bytes(b"bar"),
          total_file_size: None,
        },
      )]),
    });
  }
}
