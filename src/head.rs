use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, PartialEq)]
pub(crate) enum Head {
  Small,
  Medium(usize),
  Large(usize),
  Reserved(u8),
}

impl Head {
  pub(crate) fn encode(self, bytes: &[u8], encoder: &mut Encoder) {
    match self {
      Self::Small => {}
      Self::Medium(len) => encoder.buffer.push((0x80 + len).try_into().unwrap()),
      Self::Large(count) => {
        encoder.buffer.push((0xEF + count).try_into().unwrap());
        encoder
          .buffer
          .extend_from_slice(&bytes.len().to_le_bytes()[..count]);
      }
      Self::Reserved(value) => encoder.buffer.push(value),
    }
  }

  pub(crate) fn range(self, buffer: &[u8]) -> Result<(usize, usize), DecodeError> {
    match self {
      Self::Small => Ok((0, 1)),
      Self::Medium(len) => {
        if len == 1 {
          ensure! {
            *buffer.get(1).context(decode_error::Truncated)? > 0x7F,
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
            len > 0x6F,
            decode_error::Overlong,
          }
        }

        Ok((end, len))
      }
      Self::Reserved(value) => Err(decode_error::Reserved { value }.build()),
    }
  }
}

impl From<u8> for Head {
  fn from(head: u8) -> Self {
    match head {
      0x00..0x80 => Self::Small,
      0x80..0xF0 => Self::Medium((head - 0x80).into()),
      0xF0..0xF8 => Self::Large((head - 0xEF).into()),
      0xF8..=0xFF => Self::Reserved(head),
    }
  }
}

impl From<&[u8]> for Head {
  fn from(bytes: &[u8]) -> Self {
    if bytes.len() == 1 && bytes[0] < 0x80 {
      Head::Small
    } else if bytes.len() < 0x70 {
      Head::Medium(bytes.len())
    } else {
      Head::Large(
        bytes
          .len()
          .into_u64()
          .to_le_bytes()
          .iter()
          .rposition(|&byte| byte != 0)
          .unwrap_or_default()
          + 1,
      )
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
    for i in 0..=0xFF {
      let head = Head::from(i);
      match i {
        0x00..0x80 => assert_eq!(head, Head::Small),
        0x80..0xF0 => assert_eq!(head, Head::Medium((i - 0x80).into())),
        0xF0..0xF8 => assert_eq!(head, Head::Large((i - 0xEF).into())),
        0xF8..=0xFF => assert_eq!(head, Head::Reserved(i)),
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
}
