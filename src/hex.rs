use super::*;

pub(crate) trait Hex: DecodeOwned + Encode {
  const TAG: Tag;

  const TAGGED: bool = true;

  fn format(&self, f: &mut Formatter) -> fmt::Result {
    if Self::TAGGED {
      write!(f, "{}1", Self::TAG.prefix())?;
    }

    let buffer = self.encode_to_vec();

    write!(f, "{}", Payload(Decoder::new(&buffer).bytes().unwrap()))
  }

  fn parse(s: &str) -> Result<Self, HexError> {
    let tag = Self::TAG;

    let payload = if Self::TAGGED {
      let (actual, payload) = s.split_once('1').context(hex_error::TagMissing { tag })?;

      ensure! {
        actual == tag.prefix(),
        hex_error::UnexpectedTag { actual, expected: tag }
      }

      payload
    } else {
      s
    };

    let digits = payload
      .chars()
      .map(|digit| match digit {
        'a'..='f' => Ok(u8::try_from(digit).unwrap() - b'a' + 10),
        '0'..='9' => Ok(u8::try_from(digit).unwrap() - b'0'),
        _ => Err(HexError::Digit { digit, tag }),
      })
      .collect::<Result<Vec<u8>, HexError>>()?;

    let (chunks, remainder) = digits.as_chunks::<2>();

    ensure! {
      remainder.is_empty(),
      hex_error::OddLength { len: digits.len(), tag },
    }

    let buffer = chunks
      .iter()
      .map(|[high, low]| high << 4 | low)
      .collect::<Vec<u8>>();

    let buffer = Encoder::frame(buffer);

    Self::decode_from_slice(&buffer).context(hex_error::Decode { tag })
  }
}

struct Payload<'a>(&'a [u8]);

impl Display for Payload<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for byte in self.0 {
      write!(f, "{byte:02x}")?;
    }
    Ok(())
  }
}

pub fn encode(bytes: &[u8]) -> String {
  Payload(bytes).to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encode_bytes() {
    assert_eq!(encode(&[]), "");
    assert_eq!(encode(&[0x00, 0xab, 0xff]), "00abff");
  }

  #[test]
  fn parse_errors() {
    #[track_caller]
    fn case(s: &str, err: &str) {
      assert_eq!(s.parse::<Fingerprint>().unwrap_err().to_string(), err);
    }

    let zeros = "0".repeat(64);

    case("", "package fingerprint missing tag `package1…`");
    case("package", "package fingerprint missing tag `package1…`");
    case(
      &format!("public1{zeros}"),
      "expected package fingerprint with tag `package1…` but found `public1…`",
    );
    case(
      &format!("PACKAGE1{zeros}"),
      "expected package fingerprint with tag `package1…` but found `PACKAGE1…`",
    );
    case(
      "package10g",
      "package fingerprint contains invalid hex digit `g`",
    );
    case(
      "package1AB",
      "package fingerprint contains invalid hex digit `A`",
    );
    case(
      "package1\n",
      "package fingerprint contains invalid hex digit `\\n`",
    );
    case(
      "package1é",
      "package fingerprint contains invalid hex digit `\\u{e9}`",
    );
    case(
      "package1abc",
      "package fingerprint has odd number of hex digits: 3",
    );
    case("package1", "failed to decode package fingerprint");

    assert_matches!(
      "package1abc".parse::<Fingerprint>(),
      Err(HexError::OddLength {
        len: 3,
        tag: Tag::Fingerprint,
      }),
    );

    assert_matches!(
      "package1".parse::<Fingerprint>(),
      Err(HexError::Decode {
        source: DecodeError::ArrayLength {
          actual: 0,
          expected: 32,
        },
        tag: Tag::Fingerprint,
      }),
    );

    assert_matches!(
      format!("package1{}", &zeros[2..]).parse::<Fingerprint>(),
      Err(HexError::Decode {
        source: DecodeError::ArrayLength {
          actual: 31,
          expected: 32,
        },
        tag: Tag::Fingerprint,
      }),
    );

    assert_matches!(
      format!("package1{zeros}00").parse::<Fingerprint>(),
      Err(HexError::Decode {
        source: DecodeError::ArrayLength {
          actual: 33,
          expected: 32,
        },
        tag: Tag::Fingerprint,
      }),
    );
  }

  #[test]
  fn round_trip() {
    #[track_caller]
    fn case<T: Display + FromStr>(s: &str)
    where
      T::Err: Debug,
    {
      assert_eq!(s.parse::<T>().unwrap().to_string(), s);
    }

    case::<DisplayPrivateKey>(test::PRIVATE_KEY);
    case::<Fingerprint>(test::FINGERPRINT);
    case::<Hash>(test::HASH);
    case::<PublicKey>(test::PUBLIC_KEY);
    case::<Signature>(test::SIGNATURE);
  }
}
