use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, SerializeDisplay)]
#[cbor(transparent, validate)]
pub(crate) struct CheckedUrl(String);

impl CheckedUrl {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }

  fn check(s: &str) -> Result<(), UrlError> {
    let url = s.parse::<Url>()?;

    let scheme = url.scheme();

    ensure! {
      matches!(scheme, "http" | "https"),
      url_error::Scheme { scheme },
    }

    Ok(())
  }
}

impl FromStr for CheckedUrl {
  type Err = UrlError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::check(s)?;

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
    Self::check(self.as_str()).context(decode_error::Url)
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
      Err(DecodeError::Url {
        source: UrlError::Parse { .. }
      }),
    );

    assert_matches!(
      CheckedUrl::decode(&mut Decoder::new(&"ftp://example.com".encode_to_vec())),
      Err(DecodeError::Url {
        source: UrlError::Scheme { .. }
      }),
    );
  }

  #[test]
  fn encoding() {
    assert_cbor_eq(
      "http://example.com".parse::<CheckedUrl>().unwrap(),
      "http://example.com",
    );
  }

  #[test]
  fn scheme() {
    #[track_caller]
    fn accepted(s: &str) {
      assert_eq!(s.parse::<CheckedUrl>().unwrap().as_str(), s);
    }

    #[track_caller]
    fn rejected(s: &str, scheme: &str) {
      assert_eq!(
        s.parse::<CheckedUrl>().unwrap_err(),
        UrlError::Scheme {
          scheme: scheme.into()
        },
      );
    }

    accepted("http://example.com");
    accepted("https://example.com");
    accepted("HTTPS://example.com");

    rejected("ftp://example.com", "ftp");
    rejected("javascript:alert(1)", "javascript");
    rejected("mailto:foo@example.com", "mailto");
    rejected("file:///foo", "file");
  }
}
