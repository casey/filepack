use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, PartialEq)]
pub(crate) struct Entry {
  pub(crate) ty: EntryType,
  pub(crate) hash: Hash,
  pub(crate) size: u64,
}

impl Entry {
  pub(crate) fn new(entry: &crate::Entry) -> Self {
    match entry {
      crate::Entry::File(file) => Self {
        hash: file.hash,
        size: file.size,
        ty: EntryType::File,
      },
      crate::Entry::Directory(dir) => {
        let (hash, size) = Directory::hash(dir);
        Self {
          size,
          hash,
          ty: EntryType::Directory,
        }
      }
    }
  }
}

#[cfg(test)]
impl Decode for Entry {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.map::<u8>()?;

    let ty = decoder.key(0)?.unwrap();
    let hash = decoder.key(1)?.unwrap();
    let size = decoder.key(2)?.unwrap();

    decoder.finish()?;

    Ok(Self { ty, hash, size })
  }
}

impl Encode for Entry {
  fn encode(&self, encoder: &mut Encoder) {
    let mut encoder = encoder.map::<u8>(3);
    encoder.item(0, self.ty);
    encoder.item(1, self.hash);
    encoder.item(2, self.size);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(
      Entry {
        ty: EntryType::File,
        size: 100,
        hash: Hash::bytes(b"foo"),
      },
      &[
        0xA3, 0x00, 0x00, 0x01, 0x58, 0x20, 0x04, 0xE0, 0xBB, 0x39, 0xF3, 0x0B, 0x1A, 0x3F, 0xEB,
        0x89, 0xF5, 0x36, 0xC9, 0x3B, 0xE1, 0x50, 0x55, 0x48, 0x2D, 0xF7, 0x48, 0x67, 0x4B, 0x00,
        0xD2, 0x6E, 0x5A, 0x75, 0x77, 0x77, 0x02, 0xE9, 0x02, 0x18, 0x64,
      ],
    );
  }
}
