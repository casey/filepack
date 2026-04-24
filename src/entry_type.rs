use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, FromRepr, PartialEq)]
#[repr(u8)]
pub(crate) enum EntryType {
  File = 0,
  Directory = 1,
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
