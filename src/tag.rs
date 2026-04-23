use super::*;

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Tag(String);

impl FromStr for Tag {
  type Err = TagError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if re::TAG.is_match(s) {
      Ok(Self(s.into()))
    } else {
      Err(TagError::Parse)
    }
  }
}

impl Serialize for Tag {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.0)
  }
}

impl Decode for Tag {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.text()?.parse().context(decode_error::Tag)
  }
}

impl Encode for Tag {
  fn encode(&self, encoder: &mut Encoder) {
    self.0.encode(encoder);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_error() {
    assert_matches!(
      Tag::decode(&mut Decoder::new("".encode_to_vec())),
      Err(DecodeError::Tag {
        source: TagError::Parse,
      }),
    );
  }

  #[test]
  fn encoding() {
    assert_cbor("A0".parse::<Tag>().unwrap(), &"A0".encode_to_vec());
  }

  #[test]
  fn invalid() {
    #[track_caller]
    fn case(s: &str) {
      assert_eq!(s.parse::<Tag>().unwrap_err(), TagError::Parse);
    }

    case("");
    case("A.");
    case("A.B.");
  }

  #[test]
  fn valid() {
    #[track_caller]
    fn case(s: &str) {
      s.parse::<Tag>().unwrap();
    }

    case("A");
    case("A0");
    case("A0.A0");
    case("A0.A0.A0");
  }
}
