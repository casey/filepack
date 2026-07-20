use super::*;

#[derive(
  Clone,
  Debug,
  Decode,
  Default,
  DeserializeFromStr,
  Encode,
  Eq,
  Ord,
  PartialEq,
  PartialOrd,
  SerializeDisplay,
)]
#[cbor(transparent, validate)]
pub struct Text(String);

impl Text {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }

  fn check(s: &str) -> Result<(), TextError> {
    for character in s.chars() {
      if character != '\n' && character.is_control() {
        return Err(TextError::Control { character });
      }
    }

    Ok(())
  }

  pub(crate) fn new() -> Self {
    Self::default()
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
  }

  #[test]
  fn encoding() {
    assert_cbor_eq("foo".parse::<Text>().unwrap(), "foo");
  }

  #[test]
  fn newline_allowed() {
    assert_eq!("foo\nbar".parse::<Text>().unwrap().as_str(), "foo\nbar");
    assert_eq!("foo".parse::<Text>().unwrap().as_str(), "foo");
  }
}
