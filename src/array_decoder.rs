use super::*;

pub(crate) struct ArrayDecoder<'a> {
  decoder: Decoder<'a>,
}

impl<'a> ArrayDecoder<'a> {
  pub(crate) fn element(&mut self) -> Result<&mut Decoder<'a>, DecodeError> {
    ensure!(!self.decoder.is_empty(), decode_error::MissingElement);
    Ok(&mut self.decoder)
  }

  pub(crate) fn finish(&mut self) -> Result<(), DecodeError> {
    ensure!(self.decoder.is_empty(), decode_error::UnconsumedElements);
    Ok(())
  }

  pub(crate) fn item<T: Decode>(&mut self) -> Result<T, DecodeError> {
    T::decode(self.element()?)
  }

  pub(crate) fn new(decoder: Decoder<'a>) -> Self {
    Self { decoder }
  }

  pub(crate) fn next<T: Decode>(&mut self) -> Result<Option<T>, DecodeError> {
    if self.decoder.is_empty() {
      return Ok(None);
    }

    Ok(Some(T::decode(&mut self.decoder)?))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bounded_payload() {
    let mut decoder = Decoder::new(&[0x81, 0x82, 0, 0]);
    let mut array = decoder.array().unwrap();
    assert_matches!(array.next::<Vec<u8>>(), Err(DecodeError::Truncated));
    assert_eq!(decoder.integer().unwrap(), 0);
    assert_eq!(decoder.integer().unwrap(), 0);
    decoder.finish().unwrap();
  }

  #[test]
  fn element() {
    let mut decoder = Decoder::new(&[0x2a]);
    let mut array = decoder.array().unwrap();
    assert_matches!(array.element().unwrap().integer(), Ok(42));
    array.finish().unwrap();
  }

  #[test]
  fn item() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
    let mut array = decoder.array().unwrap();
    assert_matches!(array.item::<u64>(), Ok(0));
    assert_matches!(array.item::<u64>(), Ok(42));
    array.finish().unwrap();
  }

  #[test]
  fn missing_element() {
    let mut decoder = Decoder::new(&[0x00]);
    let mut array = decoder.array().unwrap();
    array.item::<u64>().unwrap();
    assert_matches!(array.item::<u64>(), Err(DecodeError::MissingElement));
  }

  #[test]
  fn next() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
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
