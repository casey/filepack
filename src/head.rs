use super::*;

/// The first byte of a deco-encoded byte string.
///
/// The encoding of a string depends its length and value:
///
/// - small: a single byte whose value is in [0, 127]
/// - medium: a string of length [0, 111]
/// - large: a string with length [0, 2^64-1]
///
/// Eight head values are reserved for future extensions.
///
/// Head byte ranges for the three string lengths and reserved values:
///
/// ```text
/// - 00-7F: literal single byte (0-127)
/// - 80-EF: `value - 0x80` bytes follow (0-111 bytes)
/// - F0-F7: `value - 0xEF` length bytes (1-8 length bytes)
/// - F8-FF: eight reserved values
/// ```
///
/// Strings must be encoded with the smallest head byte possible.
#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, PartialEq)]
pub(crate) enum Head {
  Small,
  Medium(usize),
  Large(usize),
  Reserved(u8),
}

impl Head {
  pub(crate) fn new(len: usize, first: Option<u8>) -> Self {
    if len == 1 && first.is_some_and(|byte| byte < 0x80) {
      Self::Small
    } else if len < 0x70 {
      Self::Medium(len)
    } else {
      Self::Large(
        len
          .into_u64()
          .to_le_bytes()
          .iter()
          .rposition(|&byte| byte != 0)
          .unwrap_or_default()
          + 1,
      )
    }
  }

  pub(crate) fn range(self, buffer: &[u8]) -> DecodeResult<(usize, usize)> {
    match self {
      Self::Small => Ok((0, 1)),
      Self::Medium(len) => {
        if len == 1 {
          ensure! {
            *buffer.get(1).context(decode_error::Truncated)? > 0x7f,
            decode_error::Overlong,
          }
        }
        Ok((1, len))
      }
      Self::Large(count) => {
        let end = count + 1;
        let mut len = [0; 8];
        len[..count].copy_from_slice(buffer.get(1..end).context(decode_error::Truncated)?);

        ensure! {
          len[count - 1] != 0,
          decode_error::Overlong,
        }

        let len = usize::try_from(u64::from_le_bytes(len))
          .ok()
          .context(decode_error::Truncated)?;

        if count == 1 {
          ensure! {
            len > 0x6f,
            decode_error::Overlong,
          }
        }

        Ok((end, len))
      }
      Self::Reserved(value) => Err(DecodeError::Reserved { value }),
    }
  }
}

impl From<u8> for Head {
  fn from(head: u8) -> Self {
    match head {
      0x00..0x80 => Self::Small,
      0x80..0xf0 => Self::Medium((head - 0x80).into()),
      0xf0..0xf8 => Self::Large((head - 0xef).into()),
      0xf8..=0xff => Self::Reserved(head),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    #[track_caller]
    fn case(bytes: &[u8], prefix: &[u8]) {
      let mut encoder = Encoder::new();
      encoder.bytes(bytes);
      let encoded = encoder.finish();
      assert_eq!(encoded, [prefix, bytes].concat());
      let mut decoder = Decoder::new(&encoded);
      assert_eq!(decoder.bytes().unwrap(), bytes);
      decoder.finish().unwrap();
    }

    case(&[], &[0x80]);
    for byte in 0..=0xff {
      case(&[byte], if byte < 0x80 { &[] } else { &[0x81] });
    }
    case(&[0; 111], &[0xef]);
    case(&[0; 112], &[0xf0, 112]);
    case(&[0; 255], &[0xf0, 255]);
    case(&[0; 256], &[0xf1, 0, 1]);
    case(&vec![0; 65535], &[0xf1, 255, 255]);
    case(&vec![0; 65536], &[0xf2, 0, 0, 1]);
  }

  #[test]
  fn head_parse() {
    for i in 0..=0xff {
      let head = Head::from(i);
      match i {
        0x00..0x80 => assert_eq!(head, Head::Small),
        0x80..0xf0 => assert_eq!(head, Head::Medium((i - 0x80).into())),
        0xf0..0xf8 => assert_eq!(head, Head::Large((i - 0xef).into())),
        0xf8..=0xff => assert_eq!(head, Head::Reserved(i)),
      }
    }
  }

  #[test]
  fn invalid() {
    #[track_caller]
    fn case(bytes: &[u8], expected: &str) {
      assert_eq!(
        Decoder::new(bytes).bytes().unwrap_err().to_string(),
        expected
      );
    }

    case(&[0x81], "truncated");
    case(&[0x81, 0x7f], "overlong encoding");
    case(&[0xf0], "truncated");
    case(&[0xf0, 0], "overlong encoding");
    case(&[0xf0, 111], "overlong encoding");
    case(&[0xf0, 112], "truncated");
    case(&[0xf1, 112, 0], "overlong encoding");
    case(&[0xf7, 255, 255, 255, 255, 255, 255, 255, 255], "truncated");
    for byte in 0xf8..=0xff {
      case(&[byte], &format!("reserved byte {byte}"));
    }
  }

  #[test]
  fn new() {
    #[track_caller]
    fn case(len: usize, first: Option<u8>, expected: Head) {
      assert_eq!(Head::new(len, first), expected);
    }

    case(0, None, Head::Medium(0));
    case(1, Some(0x00), Head::Small);
    case(1, Some(0x7f), Head::Small);
    case(1, Some(0x80), Head::Medium(1));
    case(1, Some(0xff), Head::Medium(1));
    case(2, Some(0x00), Head::Medium(2));
    case(0x6f, Some(0x00), Head::Medium(0x6f));
    case(0x70, Some(0x00), Head::Large(1));
    case(0xff, Some(0x00), Head::Large(1));
    case(0x100, Some(0x00), Head::Large(2));
    case(0xffff, Some(0x00), Head::Large(2));
    case(0x1_0000, Some(0x00), Head::Large(3));
    case(0xff_ffff, Some(0x00), Head::Large(3));
    case(0x100_0000, Some(0x00), Head::Large(4));
    case(0xffff_ffff, Some(0x00), Head::Large(4));
    case(0x1_0000_0000, Some(0x00), Head::Large(5));
    case(0xff_ffff_ffff, Some(0x00), Head::Large(5));
    case(0x100_0000_0000, Some(0x00), Head::Large(6));
    case(0xffff_ffff_ffff, Some(0x00), Head::Large(6));
    case(0x1_0000_0000_0000, Some(0x00), Head::Large(7));
    case(0xff_ffff_ffff_ffff, Some(0x00), Head::Large(7));
    case(0x100_0000_0000_0000, Some(0x00), Head::Large(8));
    case(usize::MAX, Some(0x00), Head::Large(8));
  }

  #[test]
  fn range() {
    #[track_caller]
    fn case(head: Head, buffer: &[u8], expected: Result<(usize, usize), &str>) {
      assert_eq!(
        head.range(buffer).map_err(|err| err.to_string()),
        expected.map_err(str::to_string),
      );
    }

    #[track_caller]
    fn large(count: u8, len: u64, expected: Result<(usize, usize), &str>) {
      let buffer = [
        &[0xef + count][..],
        &len.to_le_bytes()[..usize::from(count)],
      ]
      .concat();
      case(Head::Large(count.into()), &buffer, expected);
    }

    case(Head::Small, &[0x00], Ok((0, 1)));
    case(Head::Medium(0), &[0x80], Ok((1, 0)));
    case(Head::Medium(1), &[0x81], Err("truncated"));
    case(Head::Medium(1), &[0x81, 0x7f], Err("overlong encoding"));
    case(Head::Medium(1), &[0x81, 0x80], Ok((1, 1)));
    case(Head::Medium(2), &[0x82], Ok((1, 2)));
    case(Head::Medium(0x6f), &[0xef], Ok((1, 0x6f)));
    case(Head::Large(1), &[0xf0], Err("truncated"));
    case(
      Head::Large(8),
      &[0xf7, 0, 0, 0, 0, 0, 0, 0],
      Err("truncated"),
    );
    case(Head::Reserved(0xf8), &[0xf8], Err("reserved byte 248"));

    large(1, 0x00, Err("overlong encoding"));
    large(1, 0x6f, Err("overlong encoding"));
    large(1, 0x70, Ok((2, 0x70)));
    large(1, 0xff, Ok((2, 0xff)));
    large(2, 0xff, Err("overlong encoding"));
    large(2, 0x100, Ok((3, 0x100)));
    large(2, 0xffff, Ok((3, 0xffff)));
    large(3, 0xffff, Err("overlong encoding"));
    large(3, 0x1_0000, Ok((4, 0x1_0000)));
    large(3, 0xff_ffff, Ok((4, 0xff_ffff)));
    large(4, 0xff_ffff, Err("overlong encoding"));
    large(4, 0x100_0000, Ok((5, 0x100_0000)));
    large(4, 0xffff_ffff, Ok((5, 0xffff_ffff)));
    large(5, 0xffff_ffff, Err("overlong encoding"));
    large(5, 0x1_0000_0000, Ok((6, 0x1_0000_0000)));
    large(5, 0xff_ffff_ffff, Ok((6, 0xff_ffff_ffff)));
    large(6, 0xff_ffff_ffff, Err("overlong encoding"));
    large(6, 0x100_0000_0000, Ok((7, 0x100_0000_0000)));
    large(6, 0xffff_ffff_ffff, Ok((7, 0xffff_ffff_ffff)));
    large(7, 0xffff_ffff_ffff, Err("overlong encoding"));
    large(7, 0x1_0000_0000_0000, Ok((8, 0x1_0000_0000_0000)));
    large(7, 0xff_ffff_ffff_ffff, Ok((8, 0xff_ffff_ffff_ffff)));
    large(8, 0xff_ffff_ffff_ffff, Err("overlong encoding"));
    large(8, 0x100_0000_0000_0000, Ok((9, 0x100_0000_0000_0000)));
    large(8, u64::MAX, Ok((9, usize::MAX)));
  }

  #[test]
  fn round_trip() {
    #[track_caller]
    fn case(len: usize) {
      let mut encoder = Encoder::new();
      encoder.head(len);
      let mut bytes = encoder.finish();
      bytes.push(0x80);
      assert_eq!(
        Head::from(bytes[0]).range(&bytes).unwrap(),
        (bytes.len() - 1, len),
      );
    }

    for shift in 0..usize::BITS {
      let len = 1 << shift;
      case(len - 1);
      case(len);
    }
    case(0x6f);
    case(0x70);
    case(usize::MAX);
  }
}
