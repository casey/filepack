use super::*;

#[derive(
  Clone, Debug, Decode, DeserializeFromStr, Encode, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay,
)]
#[cbor(transparent, validate)]
pub struct Text(String);

impl Text {
  pub(crate) fn as_str(&self) -> &str {
    self
  }

  fn check(s: &str) -> Result<(), TextError> {
    ensure! {
      !s.is_empty(),
      text_error::Empty,
    }

    for character in s.chars() {
      ensure! {
        !character.is_control(),
        text_error::Control { character },
      }
    }

    ensure! {
      !s.starts_with(char::is_whitespace) && !s.ends_with(char::is_whitespace),
      text_error::Whitespace,
    }

    Ok(())
  }
}

impl Deref for Text {
  type Target = str;

  fn deref(&self) -> &str {
    &self.0
  }
}

impl FromStr for Text {
  type Err = TextError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::check(s)?;

    Ok(Self(s.into()))
  }
}

impl Display for Text {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Validate for Text {
  fn validate(&self) -> Result<(), DecodeError> {
    Self::check(self.as_str()).context(decode_error::Text)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check() {
    #[track_caller]
    fn case(s: &str, expected: TextError) {
      assert_eq!(Text::check(s).unwrap_err(), expected);
    }

    assert!(Text::check("foo").is_ok());

    case("foo\tbar", TextError::Control { character: '\t' });
    case("", TextError::Empty);
    case(" foo", TextError::Whitespace);
    case("foo ", TextError::Whitespace);
    case(" ", TextError::Whitespace);
    case("foo\n", TextError::Control { character: '\n' });
  }

  #[test]
  fn control() {
    #[track_caller]
    fn case(character: char) {
      assert_eq!(
        format!("foo{character}bar").parse::<Text>().unwrap_err(),
        TextError::Control { character },
      );
    }

    case('\u{00}');
    case('\u{1f}');
    case('\u{7f}');
    case('\u{9f}');
    case('\n');
    case('\r');
    case('\t');
  }

  #[test]
  fn decode_error() {
    assert_matches!(
      Text::decode(&mut Decoder::new(&"foo\tbar".encode_to_vec())),
      Err(DecodeError::Text {
        source: TextError::Control { character: '\t' }
      }),
    );

    assert_matches!(
      Text::decode(&mut Decoder::new(&"".encode_to_vec())),
      Err(DecodeError::Text {
        source: TextError::Empty,
      }),
    );

    assert_matches!(
      Text::decode(&mut Decoder::new(&" foo".encode_to_vec())),
      Err(DecodeError::Text {
        source: TextError::Whitespace,
      }),
    );

    assert_matches!(
      Text::decode(&mut Decoder::new(&"foo ".encode_to_vec())),
      Err(DecodeError::Text {
        source: TextError::Whitespace,
      }),
    );
  }

  #[test]
  fn encoding() {
    assert_cbor_eq("foo".parse::<Text>().unwrap(), "foo");
  }
}
