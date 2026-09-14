use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ServerUrl(Url);

impl Deref for ServerUrl {
  type Target = Url;

  fn deref(&self) -> &Url {
    &self.0
  }
}

impl FromStr for ServerUrl {
  type Err = UrlError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut url = if re::SCHEME.is_match(s) {
      CheckedUrl::check(s)?
    } else {
      let url = CheckedUrl::check(&format!("http://{s}"))?;

      if let Host::Domain(domain) = url.host().unwrap()
        && domain != "localhost"
      {
        CheckedUrl::check(&format!("https://{s}"))?
      } else {
        url
      }
    };

    url.path_segments_mut().unwrap().pop_if_empty().push("");

    Ok(Self(url))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[track_caller]
  pub(crate) fn case(url: &str, expected: &str) {
    assert_eq!(
      url
        .parse::<ServerUrl>()
        .unwrap_or_else(|err| panic!("failed to parse {url}: {err}"))
        .as_str(),
      expected
    );
  }

  #[test]
  fn parse() {
    case("1.1.1.1", "http://1.1.1.1/");
    case("[::1]", "http://[::1]/");
    case("foo", "https://foo/");
    case("foo:80", "https://foo:80/");
    case("föö", "https://xn--f-1gaa/");
    case("https://foo", "https://foo/");
    case("localhost", "http://localhost/");
  }

  #[test]
  fn scheme_error() {
    assert_eq!(
      "ftp://foo".parse::<ServerUrl>().unwrap_err(),
      UrlError::Scheme {
        scheme: "ftp".into()
      },
    );
  }

  #[test]
  fn trailing_slash() {
    case("https://foo/bar", "https://foo/bar/");
    case("https://foo/bar/", "https://foo/bar/");
    case("https://foo", "https://foo/");
    assert_eq!(
      "https://foo/bar"
        .parse::<ServerUrl>()
        .unwrap()
        .join("api/missing")
        .unwrap()
        .as_str(),
      "https://foo/bar/api/missing",
    );
  }
}
