use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(crate) struct Byte(pub(crate) u8);

impl Decode for Byte {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    Ok(Self(
      decoder
        .integer()?
        .try_into()
        .context(decode_error::IntegerRange)?,
    ))
  }
}

impl Encode for Byte {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.integer(self.0.into());
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(Byte(0), "00");
    assert_cbor(Byte(255), "18ff");
  }
}
