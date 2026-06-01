use super::*;

pub struct Decoder<'a> {
  buffer: &'a [u8],
  position: usize,
  stack: Vec<usize>,
}

impl<'a> Decoder<'a> {
  pub(crate) fn array<'b>(&'b mut self) -> Result<ArrayDecoder<'b, 'a>, DecodeError> {
    let len = self.expect(MajorType::Array)?;
    Ok(ArrayDecoder::new(self, len))
  }

  pub(crate) fn byte_array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
    let bytes = self.bytes()?;

    bytes.try_into().context(decode_error::ArrayLength {
      actual: bytes.len(),
      expected: N,
    })
  }

  pub(crate) fn bytes(&mut self) -> Result<&[u8], DecodeError> {
    let len = self
      .expect(MajorType::Bytes)?
      .try_into()
      .context(decode_error::SizeRange)?;

    self.raw_slice(len)
  }

  fn expect(&mut self, expected: MajorType) -> Result<u64, DecodeError> {
    let Head { major_type, value } = self.head()?;

    ensure!(
      major_type == expected,
      decode_error::UnexpectedType {
        actual: major_type,
        expected,
      }
    );

    Ok(value)
  }

  pub(crate) fn finish(self) -> Result<(), DecodeError> {
    ensure!(
      self.position == self.buffer.len(),
      decode_error::TrailingBytes
    );
    Ok(())
  }

  pub(crate) fn head(&mut self) -> Result<Head, DecodeError> {
    let initial_byte = self.raw_array::<1>()?[0];

    let major_type = MajorType::from_initial_byte(initial_byte);

    let additional_information = initial_byte & 0b11111;

    let value = match additional_information {
      0..24 => additional_information.into(),
      24 => u8::from_be_bytes(self.raw_array()?).into(),
      25 => u16::from_be_bytes(self.raw_array()?).into(),
      26 => u32::from_be_bytes(self.raw_array()?).into(),
      27 => u64::from_be_bytes(self.raw_array()?),
      value @ 28..31 => {
        return Err(decode_error::ReservedAdditionalInformation { value }.build());
      }
      value @ 31 => {
        return Err(decode_error::UnsupportedAdditionalInformation { value }.build());
      }
      32..=u8::MAX => unreachable!(),
    };

    let min = match additional_information {
      0..24 => 0,
      24 => 24,
      25 => 0x100,
      26 => 0x1_0000,
      27 => 0x1_0000_0000,
      _ => unreachable!(),
    };

    ensure!(value >= min, decode_error::OverlongInteger);

    Ok(Head { major_type, value })
  }

  pub(crate) fn integer(&mut self) -> Result<u64, DecodeError> {
    self.expect(MajorType::UnsignedInteger)
  }

  pub(crate) fn map<'b, K>(&'b mut self) -> Result<MapDecoder<'b, 'a, K>, DecodeError> {
    let len = self.expect(MajorType::Map)?;
    Ok(MapDecoder::new(self, len))
  }

  pub fn new(buffer: &'a [u8]) -> Self {
    Self {
      buffer,
      position: 0,
      stack: Vec::new(),
    }
  }

  pub(crate) fn pop_position(&mut self) {
    self.position = self.stack.pop().unwrap();
  }

  pub(crate) fn push_position(&mut self) {
    self.stack.push(self.position);
  }

  fn raw_array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
    Ok(self.raw_slice(N)?.try_into().unwrap())
  }

  fn raw_slice(&mut self, n: usize) -> Result<&[u8], DecodeError> {
    let start = self.position;
    let end = start + n;

    ensure! {
      end <= self.buffer.len(),
      decode_error::Truncated,
    }

    self.position = end;

    Ok(&self.buffer[start..end])
  }

  pub(crate) fn signed_integer(&mut self) -> Result<i128, DecodeError> {
    let Head { major_type, value } = self.head()?;

    match major_type {
      MajorType::UnsignedInteger => Ok(value.into()),
      MajorType::NegativeInteger => Ok(-1 - i128::from(value)),
      actual => Err(decode_error::ExpectedInteger { actual }.build()),
    }
  }

  pub(crate) fn text(&mut self) -> Result<&str, DecodeError> {
    let len = self
      .expect(MajorType::Text)?
      .try_into()
      .context(decode_error::SizeRange)?;

    str::from_utf8(self.raw_slice(len)?).context(decode_error::Unicode)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn byte_array() {
    let mut decoder = Decoder::new(&[0x42, 0x01, 0x02]);
    assert_eq!(decoder.byte_array::<2>().unwrap(), [0x01, 0x02]);
    decoder.finish().unwrap();
  }

  #[test]
  fn byte_array_length_mismatch() {
    assert_matches!(
      Decoder::new(&[0x42, 0x01, 0x02]).byte_array::<3>(),
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
    u8::decode(&mut decoder).unwrap();
    assert_matches!(decoder.finish(), Err(DecodeError::TrailingBytes));
  }

  #[test]
  fn integer_range() {
    assert!(matches!(
      u8::decode(&mut Decoder::new(&256u64.encode_to_vec())),
      Err(DecodeError::IntegerRange { .. }),
    ));
  }

  #[test]
  fn overlong_integer() {
    #[track_caller]
    fn case(bytes: &[u8]) {
      assert_matches!(
        Decoder::new(bytes).head(),
        Err(DecodeError::OverlongInteger),
      );
    }

    case(&[0x18, 0x17]);
    case(&[0x19, 0x00, 0xff]);
    case(&[0x1a, 0x00, 0x00, 0xff, 0xff]);
    case(&[0x1b, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff]);
  }

  #[test]
  fn position_stack() {
    let mut decoder = Decoder::new(&[0x01, 0x02]);
    decoder.push_position();
    assert_eq!(decoder.integer().unwrap(), 1);
    decoder.pop_position();
    assert_eq!(decoder.integer().unwrap(), 1);
  }

  #[test]
  fn reserved_additional_information() {
    assert_matches!(
      Decoder::new(&[0x1c]).head(),
      Err(DecodeError::ReservedAdditionalInformation { value }) if value == 28,
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

    case::<i32>(&[0x1a, 0x80, 0x00, 0x00, 0x00]);
    case::<i32>(&[0x3a, 0x80, 0x00, 0x00, 0x00]);
    case::<i64>(&[0x1b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    case::<i64>(&[0x3b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
  }

  #[test]
  fn signed_integer_type_mismatch() {
    assert_matches!(
      Decoder::new(&[0x60]).signed_integer(),
      Err(DecodeError::ExpectedInteger {
        actual: MajorType::Text,
      }),
    );
  }

  #[test]
  fn truncated() {
    assert_matches!(Decoder::new(&[]).head(), Err(DecodeError::Truncated));
  }

  #[test]
  fn type_mismatch() {
    assert_matches!(
      Decoder::new(&[0x60]).integer(),
      Err(DecodeError::UnexpectedType {
        expected: MajorType::UnsignedInteger,
        actual: MajorType::Text
      }),
    );
  }

  #[test]
  fn unicode() {
    assert_matches!(
      Decoder::new(&[0x62, 0xff, 0xfe]).text().map(drop),
      Err(DecodeError::Unicode { .. }),
    );
  }

  #[test]
  fn unsupported_additional_information() {
    assert_matches!(
      Decoder::new(&[0x1f]).head(),
      Err(DecodeError::UnsupportedAdditionalInformation { value }) if value == 31,
    );
  }
}
