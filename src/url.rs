use super::*;

#[derive(Clone, Debug, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub(crate) struct Url(String);

impl Url {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for Url {
  type Err = ::url::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.parse::<::url::Url>()?;
    Ok(Self(s.into()))
  }
}

impl Display for Url {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Decode for Url {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.text()?.parse::<Url>().context(decode_error::Url)
  }
}

impl Encode for Url {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_str().encode(encoder);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(
      "http://example.com".parse::<Url>().unwrap(),
      &"http://example.com".encode_to_vec(),
    );
  }

  #[test]
  fn url_is_not_normalized() {
    assert_eq!(
      "http://example.com".parse::<Url>().unwrap().as_str(),
      "http://example.com",
    );

    // an example of ::url::Url normalization
    assert_eq!(
      "http://example.com".parse::<::url::Url>().unwrap().as_str(),
      "http://example.com/",
    );
  }
}
