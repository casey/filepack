use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, PartialEq)]
pub(crate) struct Entry {
  pub(crate) ty: EntryType,
  pub(crate) hash: Hash,
  pub(crate) size: u64,
}

impl Decode for Entry {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.map::<u8>()?;

    let ty = decoder.required_key(0)?;
    let hash = decoder.required_key(1)?;
    let size = decoder.required_key(2)?;

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
