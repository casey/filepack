use super::*;

#[derive(Clone, Debug, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub(crate) struct CheckedUrl(String);

impl CheckedUrl {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for CheckedUrl {
  type Err = ::url::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.parse::<Url>()?;
    Ok(Self(s.into()))
  }
}

impl Display for CheckedUrl {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Decode for CheckedUrl {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder
      .text()?
      .parse::<CheckedUrl>()
      .context(decode_error::Url)
  }
}

impl Encode for CheckedUrl {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_str().encode(encoder);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_error() {
    assert_matches!(
      CheckedUrl::decode(&mut Decoder::new(&"foo".encode_to_vec())),
      Err(DecodeError::Url { .. }),
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      "http://example.com".parse::<CheckedUrl>().unwrap(),
      &"http://example.com".encode_to_vec(),
    );
  }

  #[test]
  fn url_is_not_normalized() {
    assert_eq!(
      "http://example.com".parse::<CheckedUrl>().unwrap().as_str(),
      "http://example.com",
    );

    // an example of ::url::Url normalization
    assert_eq!(
      "http://example.com".parse::<::url::Url>().unwrap().as_str(),
      "http://example.com/",
    );
  }
}
