use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, SerializeDisplay)]
#[cbor(transparent, validate)]
pub(crate) struct CheckedUrl(String);

impl CheckedUrl {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for CheckedUrl {
  type Err = url::ParseError;

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

impl Validate for CheckedUrl {
  fn validate(&self) -> Result<(), DecodeError> {
    self.as_str().parse::<Url>().context(decode_error::Url)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn checked_url_is_not_normalized() {
    assert_eq!(
      "http://example.com".parse::<CheckedUrl>().unwrap().as_str(),
      "http://example.com",
    );

    // an example of url::Url normalization
    assert_eq!(
      "http://example.com".parse::<Url>().unwrap().as_str(),
      "http://example.com/",
    );
  }

  #[test]
  fn decode_error() {
    assert_matches!(
      CheckedUrl::decode(&mut Decoder::new(&"foo".encode_to_vec())),
      Err(DecodeError::Url { .. }),
    );
  }

  #[test]
  fn encoding() {
    assert_cbor_eq(
      "http://example.com".parse::<CheckedUrl>().unwrap(),
      "http://example.com",
    );
  }
}
