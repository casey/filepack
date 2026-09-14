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
    let mut url = CheckedUrl::check(s)?;

    url.path_segments_mut().unwrap().pop_if_empty().push("");

    Ok(Self(url))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn scheme() {
    assert_eq!(
      "ftp://foo".parse::<ServerUrl>().unwrap_err(),
      UrlError::Scheme {
        scheme: "ftp".into()
      },
    );
  }

  #[test]
  fn trailing_slash() {
    assert_eq!(
      "https://foo/bar".parse::<ServerUrl>().unwrap().as_str(),
      "https://foo/bar/",
    );
    assert_eq!(
      "https://foo/bar/".parse::<ServerUrl>().unwrap().as_str(),
      "https://foo/bar/",
    );
    assert_eq!(
      "https://foo".parse::<ServerUrl>().unwrap().as_str(),
      "https://foo/",
    );
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
