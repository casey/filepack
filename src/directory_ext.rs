use super::*;

pub trait DirectoryExt {
  fn cbor(&self) -> (Vec<u8>, Hash);

  fn entry(&self) -> Entry;

  fn insert_directory(&mut self, name: &str, directory: &Directory) -> &mut Self;

  fn insert_entry(&mut self, name: &str, entry: Entry) -> &mut Self;

  fn insert_file(&mut self, name: &str, contents: &[u8]) -> &mut Self;
}

impl DirectoryExt for Directory {
  fn cbor(&self) -> (Vec<u8>, Hash) {
    let cbor = self.encode_to_vec();
    let hash = Hash::bytes(&cbor);
    (cbor, hash)
  }

  fn entry(&self) -> Entry {
    let cbor = self.encode_to_vec();

    Entry::directory(
      Hash::bytes(&cbor),
      cbor.len().into_u64(),
      self.totals().unwrap(),
    )
  }

  fn insert_directory(&mut self, name: &str, directory: &Directory) -> &mut Self {
    self.insert_entry(name, directory.entry())
  }

  fn insert_entry(&mut self, name: &str, entry: Entry) -> &mut Self {
    assert!(self.entries.insert(name.parse().unwrap(), entry).is_none());
    self
  }

  fn insert_file(&mut self, name: &str, contents: &[u8]) -> &mut Self {
    self.insert_entry(
      name,
      Entry::file(Hash::bytes(contents), contents.len().into_u64()),
    )
  }
}
