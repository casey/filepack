use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, IntoStaticStr, Display)]
#[strum(serialize_all = "kebab-case")]
pub enum EntryType {
  #[n(0)]
  File,
  #[n(1)]
  Directory,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(EntryType::File, &[0x00]);
    assert_cbor(EntryType::Directory, &[0x01]);
  }
}
