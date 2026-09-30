use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Encode, PartialEq, SerializeDisplay)]
#[deco(transparent)]
pub(crate) struct CheckedUrl(String);

impl FromStr for CheckedUrl {
  type Err = UrlError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    ensure!(!s.is_empty(), url_error::Empty);

    ensure! {
      !s.chars().any(|c| c.is_ascii_whitespace()),
      url_error::Whitespace,
    }

    if let Some(character) = s.chars().find(|c| c.is_control()) {
      return Err(UrlError::Control { character });
    }

    let (scheme, rest) = s.split_once(':').context(url_error::MissingScheme)?;

    ensure! {
      scheme.starts_with(|c: char| c.is_ascii_lowercase())
        && scheme
          .chars()
          .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '+' | '.' | '-')),
      url_error::Scheme { scheme },
    }

    ensure!(
      matches!(scheme, "http" | "https"),
      url_error::UnknownScheme { scheme }
    );

    let authority = rest
      .strip_prefix("//")
      .context(url_error::MissingAuthority { scheme })?;

    ensure!(
      authority.find(['/', '?', '#']).unwrap_or(authority.len()) > 0,
      url_error::MissingAuthority { scheme },
    );

    Ok(Self(s.into()))
  }
}

impl Display for CheckedUrl {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Decode<'_> for CheckedUrl {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    match decoder.text()?.parse() {
      Ok(url) => Ok(url),
      Err(UrlError::UnknownScheme { scheme }) => {
        Err(decoder.unknown(UnknownError::Scheme { scheme }))
      }
      Err(source) => Err(MalformedError::Url { source }.into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_error() {
    assert_matches!(
      CheckedUrl::decode(&mut Decoder::strict(&"foo".encode_to_vec())),
      Err(DecodeError::Malformed(MalformedError::Url {
        source: UrlError::MissingScheme,
      })),
    );

    let bytes = "ftp://example.com".encode_to_vec();

    assert_matches!(
      CheckedUrl::decode(&mut Decoder::new(&bytes)),
      Err(DecodeError::Unknown {
        source: UnknownError::Scheme { scheme },
        strict: false,
      }) if scheme == "ftp",
    );

    assert_matches!(
      CheckedUrl::decode(&mut Decoder::strict(&bytes)),
      Err(DecodeError::Unknown {
        source: UnknownError::Scheme { scheme },
        strict: true,
      }) if scheme == "ftp",
    );
  }

  #[test]
  fn encoding() {
    assert_deco_eq(
      "http://example.com".parse::<CheckedUrl>().unwrap(),
      "http://example.com",
    );
  }

  #[test]
  fn parse() {
    #[track_caller]
    fn err(s: &str, expected: UrlError) {
      assert_eq!(s.parse::<CheckedUrl>().unwrap_err(), expected);
    }

    #[track_caller]
    fn ok(s: &str) {
      assert_eq!(s.parse::<CheckedUrl>().unwrap().to_string(), s);
    }

    ok("http://example.com");
    ok("https://föö.example/道");

    err("", UrlError::Empty);
    err("http://example.com/ foo", UrlError::Whitespace);
    err(
      "http://example.com/\u{85}foo",
      UrlError::Control {
        character: '\u{85}',
      },
    );
    err("foo", UrlError::MissingScheme);
    err(
      "HTTPS://example.com",
      UrlError::Scheme {
        scheme: "HTTPS".into(),
      },
    );
    err(
      "ht_tp://example.com",
      UrlError::Scheme {
        scheme: "ht_tp".into(),
      },
    );
    err(
      "ftp://example.com",
      UrlError::UnknownScheme {
        scheme: "ftp".into(),
      },
    );
    err(
      "http:foo",
      UrlError::MissingAuthority {
        scheme: "http".into(),
      },
    );
    err(
      "https://",
      UrlError::MissingAuthority {
        scheme: "https".into(),
      },
    );
  }
}
