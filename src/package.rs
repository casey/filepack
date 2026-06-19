use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Encode, Decode, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Package {
  #[n(0)]
  pub(crate) creator: Option<ComponentBuf>,
  #[n(1)]
  pub(crate) date: Option<DateTime>,
  #[n(2)]
  pub(crate) description: Option<String>,
  #[n(3)]
  pub(crate) homepage: Option<CheckedUrl>,
  #[n(4)]
  pub(crate) nfo: Option<filename::Nfo>,
  #[n(5)]
  pub(crate) title: Option<ComponentBuf>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Package {
      creator: Some("foo".parse().unwrap()),
      date: Some("2024-01-01".parse().unwrap()),
      description: Some("bar".into()),
      homepage: Some("http://example.com".parse().unwrap()),
      nfo: Some("info.nfo".parse().unwrap()),
      title: Some("foo-A0".parse().unwrap()),
    });
  }
}
