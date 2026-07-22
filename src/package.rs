use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Encode, Decode, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Package {
  #[n(0)]
  pub(crate) creator: Option<ComponentBuf>,
  #[n(1)]
  pub(crate) description: Option<Text>,
  #[n(2)]
  pub(crate) homepage: Option<CheckedUrl>,
  #[n(3)]
  pub(crate) readme: Option<ComponentBuf>,
  #[n(4)]
  pub(crate) time: Option<Time>,
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
      description: Some("bar".parse().unwrap()),
      homepage: Some("http://example.com".parse().unwrap()),
      readme: Some("README.md".parse().unwrap()),
      time: Some("2024-01-01".parse().unwrap()),
      title: Some("foo-A0".parse().unwrap()),
    });
  }
}
