use super::*;

#[derive(Clone)]
pub struct Decoder<'a> {
  buffer: &'a [u8],
  options: DecodeOptions,
  position: usize,
}

impl<'a> Decoder<'a> {
  pub(crate) fn array(&mut self) -> DecodeResult<ArrayDecoder<'a>> {
    Ok(ArrayDecoder::new(self.child()?))
  }

  pub(crate) fn boolean(&mut self) -> DecodeResult<bool> {
    match self.integer()? {
      0 => Ok(false),
      1 => Ok(true),
      value => Err(DecodeError::Boolean { value }),
    }
  }

  pub(crate) fn byte_array<const N: usize>(&mut self) -> DecodeResult<[u8; N]> {
    let bytes = self.bytes()?;

    bytes.try_into().ok().context(decode_error::ArrayLength {
      actual: bytes.len(),
      expected: N,
    })
  }

  pub(crate) fn bytes(&mut self) -> DecodeResult<&'a [u8]> {
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

  fn child(&mut self) -> DecodeResult<Self> {
    Ok(Self::with_options(self.options, self.bytes()?))
  }

  pub(crate) fn empty_map<K>(&self) -> MapDecoder<'a, K> {
    MapDecoder::new(Self::with_options(self.options, &[]))
  }

  pub(crate) fn finish(self) -> DecodeResult {
    ensure!(self.is_empty(), decode_error::TrailingBytes);
    Ok(())
  }

  pub(crate) fn integer(&mut self) -> DecodeResult<u64> {
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

  pub(crate) fn magic_bytes(&mut self, expected: &MagicBytes) -> DecodeResult {
    let actual = self.bytes()?;
    ensure!(
      actual == expected,
      decode_error::MagicBytes {
        actual: &actual[..actual.len().min(expected.len() + 1)],
        expected: *expected,
      },
    );
    Ok(())
  }

  pub(crate) fn map<K>(&mut self) -> DecodeResult<MapDecoder<'a, K>> {
    Ok(MapDecoder::new(self.child()?))
  }

  pub fn new(buffer: &'a [u8]) -> Self {
    Self::with_options(DecodeOptions::new(), buffer)
  }

  pub(crate) fn signed_integer(&mut self) -> DecodeResult<i64> {
    let integer = self.integer()?;
    Ok((integer >> 1).cast_signed() ^ -(integer & 1).cast_signed())
  }

  pub(crate) fn strict(&self) -> bool {
    self.options.strict
  }

  #[cfg(test)]
  pub(crate) fn strict_array(&mut self) -> DecodeResult<ArrayDecoder<'a>> {
    Ok(ArrayDecoder::new(self.strict_child()?))
  }

  fn strict_child(&mut self) -> DecodeResult<Self> {
    let mut child = self.child()?;
    child.options.strict = true;
    Ok(child)
  }

  pub(crate) fn strict_map<K>(&mut self) -> DecodeResult<MapDecoder<'a, K>> {
    Ok(MapDecoder::new(self.strict_child()?))
  }

  pub(crate) fn text(&mut self) -> DecodeResult<&'a str> {
    str::from_utf8(self.bytes()?).context(decode_error::Unicode)
  }

  pub(crate) fn with_options(options: DecodeOptions, buffer: &'a [u8]) -> Self {
    Self {
      buffer,
      position: 0,
      options,
    }
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
      case(&[0x82, 0x80, 0], signed, "overlong integer");
      case(
        &[0x89, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        signed,
        "integer exceeds eight bytes",
      );
    }
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
    fn case<'a, T: Debug + Decode<'a>>(bytes: &'a [u8]) {
      assert_matches!(
        T::decode_from_slice(bytes),
        Err(DecodeError::IntegerRange { .. }),
      );
    }

    case::<i32>(&[0x85, 0x00, 0x00, 0x00, 0x00, 0x01]);
    case::<i32>(&[0x85, 0x01, 0x00, 0x00, 0x00, 0x01]);
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
