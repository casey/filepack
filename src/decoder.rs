use super::*;

#[derive(Clone)]
pub struct Decoder<'a> {
  buffer: &'a [u8],
  position: usize,
}

impl<'a> Decoder<'a> {
  pub(crate) fn array(&mut self) -> Result<ArrayDecoder<'a>, DecodeError> {
    Ok(ArrayDecoder::new(Self::new(self.bytes()?)))
  }

  pub(crate) fn boolean(&mut self) -> Result<bool, DecodeError> {
    match self.integer()? {
      0 => Ok(false),
      1 => Ok(true),
      value => Err(decode_error::Boolean { value }.build()),
    }
  }

  pub(crate) fn byte_array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
    let bytes = self.bytes()?;

    bytes.try_into().context(decode_error::ArrayLength {
      actual: bytes.len(),
      expected: N,
    })
  }

  pub(crate) fn bytes(&mut self) -> Result<&'a [u8], DecodeError> {
    let head = Head::from(
      *self
        .buffer
        .get(self.position)
        .context(decode_error::Truncated)?,
    );
    let (start, len) = head.range(&self.buffer[self.position..])?;
    let start = self.position + start;
    let end = start.checked_add(len).context(decode_error::Truncated)?;
    let bytes = self
      .buffer
      .get(start..end)
      .context(decode_error::Truncated)?;
    self.position = end;
    Ok(bytes)
  }

  pub(crate) fn finish(self) -> Result<(), DecodeError> {
    ensure!(self.is_empty(), decode_error::TrailingBytes);
    Ok(())
  }

  pub(crate) fn integer(&mut self) -> Result<u64, DecodeError> {
    let bytes = self.bytes()?;
    ensure!(!bytes.is_empty(), decode_error::EmptyInteger);
    ensure!(
      bytes.len() == 1 || bytes.last() != Some(&0),
      decode_error::OverlongInteger
    );
    ensure!(bytes.len() <= 8, decode_error::IntegerLength);
    let mut value = [0; 8];
    value[..bytes.len()].copy_from_slice(bytes);
    Ok(u64::from_le_bytes(value))
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.position == self.buffer.len()
  }

  pub(crate) fn map<K>(&mut self) -> Result<MapDecoder<'a, K>, DecodeError> {
    Ok(MapDecoder::new(Self::new(self.bytes()?)))
  }

  pub fn new(buffer: &'a [u8]) -> Self {
    Self {
      buffer,
      position: 0,
    }
  }

  pub(crate) fn signed_integer(&mut self) -> Result<i128, DecodeError> {
    let bytes = self.bytes()?;
    let last = *bytes.last().context(decode_error::EmptyInteger)?;
    if bytes.len() > 1 {
      let previous = bytes[bytes.len() - 2];
      ensure!(
        !((last == 0 && previous < 0x80) || (last == 0xFF && previous >= 0x80)),
        decode_error::OverlongInteger
      );
    }
    ensure!(bytes.len() <= 8, decode_error::IntegerLength);
    let mut value = [if last < 0x80 { 0 } else { 0xFF }; 8];
    value[..bytes.len()].copy_from_slice(bytes);
    Ok(i64::from_le_bytes(value).into())
  }

  pub(crate) fn text(&mut self) -> Result<&str, DecodeError> {
    str::from_utf8(self.bytes()?).context(decode_error::Unicode)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn boolean() {
    assert_matches!(
      Decoder::new(&[0x02]).boolean(),
      Err(DecodeError::Boolean { value: 2 }),
    );
  }

  #[test]
  fn byte_array() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0x02]);
    assert_eq!(decoder.byte_array::<2>().unwrap(), [0x01, 0x02]);
    decoder.finish().unwrap();
  }

  #[test]
  fn byte_array_length_mismatch() {
    assert_matches!(
      Decoder::new(&[0x82, 0x01, 0x02]).byte_array::<3>(),
      Err(DecodeError::ArrayLength {
        actual: 2,
        expected: 3,
        ..
      }),
    );
  }

  #[test]
  fn finish_errors_on_trailing_bytes() {
    let mut decoder = Decoder::new(&[0x00, 0x00]);
    u64::decode(&mut decoder).unwrap();
    assert_matches!(decoder.finish(), Err(DecodeError::TrailingBytes));
  }

  #[test]
  fn integer_empty() {
    assert_matches!(
      Decoder::new(&[0x80]).integer(),
      Err(DecodeError::EmptyInteger),
    );
  }

  #[test]
  fn invalid_integers() {
    #[track_caller]
    fn case(bytes: &[u8], signed: bool, expected: &str) {
      let mut decoder = Decoder::new(bytes);
      let error = if signed {
        decoder.signed_integer().unwrap_err()
      } else {
        decoder.integer().unwrap_err()
      };
      assert_eq!(error.to_string(), expected);
    }

    for signed in [false, true] {
      case(&[0x80], signed, "empty integer");
      case(&[0x82, 0, 0], signed, "overlong integer");
      case(
        &[0x89, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        signed,
        "integer exceeds eight bytes",
      );
    }
    case(&[0x82, 0xff, 0xff], true, "overlong integer");
    case(&[0x82, 0x80, 0xff], true, "overlong integer");
    case(&[0x82, 0x7f, 0], true, "overlong integer");
    case(&[0x82, 0x80, 0], false, "overlong integer");
  }

  #[test]
  fn overlong_integer() {
    #[track_caller]
    fn case(bytes: &[u8]) {
      assert_matches!(
        Decoder::new(bytes).integer(),
        Err(DecodeError::OverlongInteger),
      );
    }

    case(&[0x82, 0x00, 0x00]);
    case(&[0x82, 0x01, 0x00]);
    case(&[0x83, 0xff, 0xff, 0x00]);
  }

  #[test]
  fn reserved() {
    assert_matches!(
      Decoder::new(&[0xf8]).bytes(),
      Err(DecodeError::Reserved { value: 0xf8 }),
    );
  }

  #[test]
  fn signed_integer_empty() {
    assert_matches!(
      Decoder::new(&[0x80]).signed_integer(),
      Err(DecodeError::EmptyInteger),
    );
  }

  #[test]
  fn signed_integer_range() {
    #[track_caller]
    fn case<T: Debug + Decode>(bytes: &[u8]) {
      assert_matches!(
        T::decode_from_slice(bytes),
        Err(DecodeError::IntegerRange { .. }),
      );
    }

    case::<i32>(&[0x85, 0x00, 0x00, 0x00, 0x80, 0x00]);
    case::<i32>(&[0x85, 0xff, 0xff, 0xff, 0x7f, 0xff]);
  }

  #[test]
  fn truncated() {
    assert_matches!(
      Decoder::new(&[]).bytes().unwrap_err(),
      DecodeError::Truncated,
    );

    assert_matches!(
      Decoder::new(&[0x82, 0x01]).bytes().unwrap_err(),
      DecodeError::Truncated,
    );

    assert_matches!(
      Decoder::new(&[0xf7, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
        .bytes()
        .unwrap_err(),
      DecodeError::Truncated,
    );
  }

  #[test]
  fn unicode() {
    assert_matches!(
      Decoder::new(&[0x82, 0xff, 0xfe]).text().map(drop),
      Err(DecodeError::Unicode { .. }),
    );
  }
}
