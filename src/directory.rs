use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Default, Encode, Decode, PartialEq)]
pub(crate) struct Directory {
  #[n(0)]
  pub(crate) version: Version,
  #[n(1)]
  pub(crate) entries: BTreeMap<ComponentBuf, Entry>,
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
        },
      )]),
    });
  }
}
