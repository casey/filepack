use super::*;

pub(crate) struct ArrayDecoder<'a, 'b> {
  decoder: &'a mut Decoder<'b>,
  remaining: u64,
}

impl<'a, 'b> ArrayDecoder<'a, 'b> {
  pub(crate) fn element(&mut self) -> Result<&mut Decoder<'b>, DecodeError> {
    ensure!(self.remaining > 0, decode_error::MissingElement);
    self.remaining -= 1;
    Ok(&mut *self.decoder)
  }

  #[cfg_attr(not(test), allow(dead_code))]
  pub(crate) fn finish(&mut self) -> Result<(), DecodeError> {
    ensure!(self.remaining == 0, decode_error::UnconsumedElements);
    Ok(())
  }

  pub(crate) fn item<T: Decode>(&mut self) -> Result<T, DecodeError> {
    T::decode(self.element()?)
  }

  pub(crate) fn new(decoder: &'a mut Decoder<'b>, len: u64) -> Self {
    Self {
      decoder,
      remaining: len,
    }
  }

  pub(crate) fn next<T: Decode>(&mut self) -> Result<Option<T>, DecodeError> {
    if self.remaining == 0 {
      return Ok(None);
    }

    self.remaining -= 1;

    Ok(Some(T::decode(&mut *self.decoder)?))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn element() {
    let mut decoder = Decoder::new(&[0x81, 0x18, 0x2a]);
    let mut array = decoder.array().unwrap();
    assert_matches!(array.element().unwrap().integer(), Ok(42));
    array.finish().unwrap();
  }

  #[test]
  fn item() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x18, 0x2a]);
    let mut array = decoder.array().unwrap();
    assert_matches!(array.item::<u64>(), Ok(0));
    assert_matches!(array.item::<u64>(), Ok(42));
    array.finish().unwrap();
  }

  #[test]
  fn missing_element() {
    let mut decoder = Decoder::new(&[0x81, 0x00]);
    let mut array = decoder.array().unwrap();
    array.item::<u64>().unwrap();
    assert_matches!(array.item::<u64>(), Err(DecodeError::MissingElement));
  }

  #[test]
  fn next() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x18, 0x2a]);
    let mut array = decoder.array().unwrap();
    assert_matches!(array.next::<u64>(), Ok(Some(0)));
    assert_matches!(array.next::<u64>(), Ok(Some(42)));
    assert_matches!(array.next::<u64>(), Ok(None));
    array.finish().unwrap();
  }

  #[test]
  fn unconsumed_elements() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x01]);
    let mut array = decoder.array().unwrap();
    array.item::<u64>().unwrap();
    assert_matches!(array.finish(), Err(DecodeError::UnconsumedElements));
  }
}
