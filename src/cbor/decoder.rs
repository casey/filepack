use super::*;

pub(crate) struct Decoder {
  buffer: Vec<u8>,
  i: usize,
}

impl Decoder {
  fn array<const N: usize>(&mut self) -> [u8; N] {
    self.slice(N).try_into().unwrap()
  }

  pub(crate) fn bytes(&mut self) -> Result<&[u8], DecodeError> {
    let head = self.head()?;
    ensure!(
      head.major_type == MajorType::Bytes,
      decode_error::UnexpectedType {
        expected: MajorType::Bytes,
        actual: head.major_type,
      }
    );
    Ok(self.slice(head.value.try_into().unwrap()))
  }

  pub(crate) fn finish(self) -> Result<(), DecodeError> {
    ensure!(self.i == self.buffer.len(), decode_error::TrailingBytes);
    Ok(())
  }

  pub(crate) fn head(&mut self) -> Result<Head, DecodeError> {
    let initial_byte = self.array::<1>()[0];

    let major_type = MajorType::from_initial_byte(initial_byte);

    let additional_information = initial_byte & 0b11111;

    let (value, min) = match additional_information {
      0..24 => (additional_information.into(), 0),
      24 => (u8::from_be_bytes(self.array()).into(), u64::from(24u8)),
      25 => (
        u16::from_be_bytes(self.array()).into(),
        u64::from(u8::MAX) + 1,
      ),
      26 => (
        u32::from_be_bytes(self.array()).into(),
        u64::from(u16::MAX) + 1,
      ),
      27 => (u64::from_be_bytes(self.array()), u64::from(u32::MAX) + 1),
      value @ 28..31 => {
        return Err(decode_error::ReservedAdditionalInformation { value }.build());
      }
      value @ 31 => {
        return Err(decode_error::UnsupportedAdditionalInformation { value }.build());
      }
      32..=u8::MAX => unreachable!(),
    };

    ensure!(value >= min, decode_error::OverlongInteger);

    Ok(Head { major_type, value })
  }

  pub(crate) fn integer(&mut self) -> Result<u64, DecodeError> {
    let head = self.head()?;
    ensure!(
      head.major_type == MajorType::Integer,
      decode_error::UnexpectedType {
        expected: MajorType::Integer,
        actual: head.major_type,
      }
    );
    Ok(head.value)
  }

  pub(crate) fn map<K>(&mut self) -> Result<MapDecoder<K>, DecodeError> {
    MapDecoder::new(self)
  }

  pub(crate) fn new(buffer: Vec<u8>) -> Self {
    Self { i: 0, buffer }
  }

  fn slice(&mut self, n: usize) -> &[u8] {
    let start = self.i;
    let end = start + n;
    self.i = end;
    &self.buffer[start..end]
  }

  pub(crate) fn text(&mut self) -> Result<&str, DecodeError> {
    let head = self.head()?;

    ensure!(
      head.major_type == MajorType::Text,
      decode_error::UnexpectedType {
        expected: MajorType::Text,
        actual: head.major_type,
      }
    );

    let len = head.value.try_into().context(decode_error::SizeRange)?;

    str::from_utf8(self.slice(len)).context(decode_error::Unicode)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
    case(&[0x19, 0x00, 0xFF]);
    case(&[0x1A, 0x00, 0x00, 0xFF, 0xFF]);
    case(&[0x1B, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]);
  }

  #[test]
  fn trailing_bytes() {
    let mut decoder = Decoder::new(vec![0x00, 0x01]);
    decoder.head().unwrap();
    assert_eq!(decoder.finish(), Err(DecodeError::TrailingBytes));
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

    assert_eq!(
      Decoder::new(vec![0x00]).text(),
      Err(DecodeError::UnexpectedType {
        expected: MajorType::Text,
        actual: MajorType::Integer,
      }),
    );

    assert_eq!(
      Decoder::new(vec![0x00]).bytes(),
      Err(DecodeError::UnexpectedType {
        expected: MajorType::Bytes,
        actual: MajorType::Integer,
      }),
    );
  }

  #[test]
  fn reserved_additional_inforomation() {
    assert_eq!(
      Decoder::new(vec![0x1C]).head(),
      Err(DecodeError::ReservedAdditionalInformation { value: 28 }),
    );
  }

  #[test]
  fn unsupported_additional_inforomation() {
    assert_eq!(
      Decoder::new(vec![0x1F]).head(),
      Err(DecodeError::UnsupportedAdditionalInformation { value: 31 }),
    );
  }
}
