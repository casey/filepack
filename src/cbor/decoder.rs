use super::*;

pub(crate) struct Decoder {
  buffer: Vec<u8>,
  position: usize,
  stack: Vec<usize>,
}

impl Decoder {
  fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
    Ok(self.slice(N)?.try_into().unwrap())
  }

  pub(crate) fn bytes(&mut self) -> Result<&[u8], DecodeError> {
    let len = self
      .expect(MajorType::Bytes)?
      .try_into()
      .context(decode_error::SizeRange)?;

    self.slice(len)
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
    let initial_byte = self.array::<1>()?[0];

    let major_type = MajorType::from_initial_byte(initial_byte);

    let additional_information = initial_byte & 0b11111;

    let value = match additional_information {
      0..24 => additional_information.into(),
      24 => u8::from_be_bytes(self.array()?).into(),
      25 => u16::from_be_bytes(self.array()?).into(),
      26 => u32::from_be_bytes(self.array()?).into(),
      27 => u64::from_be_bytes(self.array()?),
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
    self.expect(MajorType::Integer)
  }

  pub(crate) fn map<K>(&mut self) -> Result<MapDecoder<K>, DecodeError> {
    let len = self.expect(MajorType::Map)?;
    Ok(MapDecoder::new(self, len))
  }

  pub(crate) fn new(buffer: Vec<u8>) -> Self {
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

  fn slice(&mut self, n: usize) -> Result<&[u8], DecodeError> {
    let start = self.position;
    let end = start + n;

    ensure! {
      end <= self.buffer.len(),
      decode_error::Truncated,
    }

    self.position = end;

    Ok(&self.buffer[start..end])
  }

  pub(crate) fn text(&mut self) -> Result<&str, DecodeError> {
    let len = self
      .expect(MajorType::Text)?
      .try_into()
      .context(decode_error::SizeRange)?;

    str::from_utf8(self.slice(len)?).context(decode_error::Unicode)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn finish_errors_on_trailing_bytes() {
    let mut decoder = Decoder::new(vec![0x00, 0x00]);
    u8::decode(&mut decoder).unwrap();
    assert_eq!(decoder.finish().unwrap_err(), DecodeError::TrailingBytes,);
  }

  #[test]
  fn integer_range() {
    assert!(matches!(
      u8::decode(&mut Decoder::new(256u64.encode_to_vec())),
      Err(DecodeError::IntegerRange { .. }),
    ));
  }

  #[test]
  fn overlong_integer() {
    #[track_caller]
    fn case(bytes: &[u8]) {
      assert_eq!(
        Decoder::new(bytes.to_vec()).head(),
        Err(DecodeError::OverlongInteger),
      );
    }

    case(&[0x18, 0x17]);
    case(&[0x19, 0x00, 0xff]);
    case(&[0x1a, 0x00, 0x00, 0xff, 0xff]);
    case(&[0x1b, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff]);
  }

  #[test]
  fn reserved_additional_information() {
    assert_eq!(
      Decoder::new(vec![0x1c]).head(),
      Err(DecodeError::ReservedAdditionalInformation { value: 28 }),
    );
  }

  #[test]
  fn truncated() {
    assert_eq!(Decoder::new(vec![]).head(), Err(DecodeError::Truncated),);
  }

  #[test]
  fn type_mismatch() {
    assert_eq!(
      Decoder::new(vec![0x60]).integer(),
      Err(DecodeError::UnexpectedType {
        expected: MajorType::Integer,
        actual: MajorType::Text,
      }),
    );
  }

  #[test]
  #[expect(invalid_from_utf8)]
  fn unicode() {
    assert_eq!(
      Decoder::new(vec![0x62, 0xff, 0xfe]).text().map(drop),
      Err(DecodeError::Unicode {
        source: str::from_utf8(&[0xff, 0xfe]).unwrap_err()
      }),
    );
  }

  #[test]
  fn unsupported_additional_information() {
    assert_eq!(
      Decoder::new(vec![0x1f]).head(),
      Err(DecodeError::UnsupportedAdditionalInformation { value: 31 }),
    );
  }
}
