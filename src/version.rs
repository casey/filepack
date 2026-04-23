use super::*;

#[derive(Clone, Copy, Debug, FromRepr, PartialEq)]
#[repr(u8)]
pub(crate) enum Version {
  Zero = 0,
}

impl Encode for Version {
  fn encode(&self, encoder: &mut Encoder) {
    (*self as u8).encode(encoder);
  }
}

impl Decode for Version {
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
    assert_cbor(Version::Zero, &[0x00]);
  }
}
