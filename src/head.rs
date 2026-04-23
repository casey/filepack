use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Head {
  pub(crate) major_type: MajorType,
  pub(crate) value: u64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    #[track_caller]
    fn case(major_type: MajorType, value: u64, expected: &[u8]) {
      let mut encoder = Encoder::new();
      encoder.head(Head { major_type, value });
      assert_eq!(encoder.finish(), expected);
    }

    case(MajorType::UnsignedInteger, 0, &[0x00]);
    case(MajorType::UnsignedInteger, 23, &[0x17]);
    case(MajorType::UnsignedInteger, 24, &[0x18, 0x18]);
    case(MajorType::UnsignedInteger, 255, &[0x18, 0xff]);
    case(MajorType::UnsignedInteger, 256, &[0x19, 0x01, 0x00]);
    case(MajorType::UnsignedInteger, 65535, &[0x19, 0xff, 0xff]);
    case(
      MajorType::UnsignedInteger,
      65536,
      &[0x1a, 0x00, 0x01, 0x00, 0x00],
    );
    case(
      MajorType::UnsignedInteger,
      4_294_967_295,
      &[0x1a, 0xff, 0xff, 0xff, 0xff],
    );
    case(
      MajorType::UnsignedInteger,
      4_294_967_296,
      &[0x1b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    );
    case(MajorType::Text, 0, &[0x60]);
    case(MajorType::Bytes, 24, &[0x58, 0x18]);
    case(MajorType::Map, 2, &[0xa2]);
  }
}
