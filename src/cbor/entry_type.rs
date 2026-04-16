use super::*;

#[derive(Clone, Copy, Debug, FromRepr, PartialEq)]
#[repr(u8)]
pub(crate) enum EntryType {
  File = 0,
  Directory = 1,
}

impl Encode for EntryType {
  fn encode(&self, encoder: &mut Encoder) {
    (*self as u8).encode(encoder);
  }
}

#[cfg(test)]
impl Decode for EntryType {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let discriminant = decoder.integer()?;
    Ok(Self::from_repr(discriminant.try_into().unwrap()).unwrap())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(EntryType::File);
    assert_encoding(EntryType::Directory);
  }
}
