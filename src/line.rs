use super::*;

/// Non-empty single line of text.
///
/// May not contain control characters, including tabs and line breaks, or
/// leading or trailing whitespace.
#[derive(
  Clone,
  Debug,
  Decode,
  DeserializeFromStr,
  Encode,
  Eq,
  Json,
  Ord,
  PartialEq,
  PartialOrd,
  SerializeDisplay,
)]
#[json(string)]
#[deco(transparent, validate)]
pub struct Line(String);

impl Line {
  pub(crate) fn as_str(&self) -> &str {
    self
  }

  fn check(s: &str) -> Result<(), LineError> {
    ensure! {
      !s.is_empty(),
      line_error::Empty,
    }

    for character in s.chars() {
      ensure! {
        !character.is_control(),
        line_error::Control { character },
      }
    }

    ensure! {
      !s.starts_with(char::is_whitespace) && !s.ends_with(char::is_whitespace),
      line_error::Whitespace,
    }

    Ok(())
  }
}

impl Deref for Line {
  type Target = str;

  fn deref(&self) -> &str {
    &self.0
  }
}

impl FromStr for Line {
  type Err = LineError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::check(s)?;

    Ok(Self(s.into()))
  }
}

impl PartialEq<str> for Line {
  fn eq(&self, s: &str) -> bool {
    self.as_str().eq(s)
  }
}

impl Display for Line {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Validate for Line {
  fn validate(&self) -> Result<(), MalformedError> {
    Self::check(self.as_str()).context(malformed_error::Line)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check() {
    #[track_caller]
    fn case(s: &str, expected: LineError) {
      assert_eq!(Line::check(s).unwrap_err(), expected);
    }

    assert!(Line::check("foo").is_ok());

    case("foo\tbar", LineError::Control { character: '\t' });
    case("", LineError::Empty);
    case(" foo", LineError::Whitespace);
    case("foo ", LineError::Whitespace);
    case(" ", LineError::Whitespace);
    case("foo\n", LineError::Control { character: '\n' });
  }

  #[test]
  fn control() {
    #[track_caller]
    fn case(character: char) {
      assert_eq!(
        format!("foo{character}bar").parse::<Line>().unwrap_err(),
        LineError::Control { character },
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
      Line::decode(&mut Decoder::strict(&"foo\tbar".encode_to_vec())),
      Err(DecodeError::Malformed(MalformedError::Line {
        source: LineError::Control { character: '\t' }
      })),
    );

    assert_matches!(
      Line::decode(&mut Decoder::strict(&"".encode_to_vec())),
      Err(DecodeError::Malformed(MalformedError::Line {
        source: LineError::Empty,
      })),
    );

    assert_matches!(
      Line::decode(&mut Decoder::strict(&" foo".encode_to_vec())),
      Err(DecodeError::Malformed(MalformedError::Line {
        source: LineError::Whitespace,
      })),
    );

    assert_matches!(
      Line::decode(&mut Decoder::strict(&"foo ".encode_to_vec())),
      Err(DecodeError::Malformed(MalformedError::Line {
        source: LineError::Whitespace,
      })),
    );
  }

  #[test]
  fn encoding() {
    assert_deco_eq("foo".parse::<Line>().unwrap(), "foo");
  }
}
