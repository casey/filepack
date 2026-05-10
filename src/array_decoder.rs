use super::*;

pub(crate) struct ArrayDecoder<'a, 'b> {
  decoder: &'a mut Decoder<'b>,
  remaining: u64,
}

impl<'a, 'b> ArrayDecoder<'a, 'b> {
  pub(crate) fn element<K: Decode>(&mut self) -> Result<K, DecodeError> {
    assert!(self.remaining > 0);
    let element = K::decode(self.decoder)?;
    self.remaining -= 1;
    Ok(element)
  }

  pub(crate) fn new(decoder: &'a mut Decoder<'b>, len: u64) -> Self {
    Self {
      decoder,
      remaining: len,
    }
  }
}
