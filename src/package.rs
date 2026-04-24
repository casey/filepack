use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Encode, Decode, PartialEq, Serialize)]
pub(crate) struct Package {
  #[n(0)]
  pub(crate) creator: Option<ComponentBuf>,
  #[n(1)]
  pub(crate) creator_tag: Option<Tag>,
  #[n(2)]
  pub(crate) date: Option<DateTime>,
  #[n(3)]
  pub(crate) description: Option<String>,
  #[n(4)]
  pub(crate) homepage: Option<Url>,
  #[n(5)]
  pub(crate) nfo: Option<filename::Nfo>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Package {
      creator: Some("foo".parse().unwrap()),
      creator_tag: Some("A0".parse().unwrap()),
      date: Some("2024-01-01".parse().unwrap()),
      description: Some("bar".into()),
      homepage: Some("http://example.com".parse().unwrap()),
      nfo: Some("info.nfo".parse().unwrap()),
    });
  }
}
