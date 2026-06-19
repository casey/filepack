use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay)]
pub struct Text(String);

impl Text {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for Text {
  type Err = TextError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for character in s.chars() {
      if character != '\n' && character.is_control() {
        return Err(TextError::Control { character });
      }
    }

    Ok(Self(s.into()))
  }
}

impl Display for Text {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Decode for Text {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.text()?.parse().context(decode_error::Text)
  }
}

impl Encode for Text {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_str().encode(encoder);
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
