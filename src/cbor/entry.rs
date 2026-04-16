use super::*;

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
    assert_encoding(Entry {
      ty: EntryType::File,
      size: 100,
      hash: Hash::bytes(b"foo"),
    });
  }
}
