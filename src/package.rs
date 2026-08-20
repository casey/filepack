use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Encode, Decode, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Package {
  #[n(0)]
  pub(crate) colophon: Option<RelativePath>,
  #[n(1)]
  pub(crate) creator: Option<ComponentBuf>,
  #[n(2)]
  pub(crate) description: Option<Text>,
  #[n(3)]
  pub(crate) homepage: Option<CheckedUrl>,
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
      colophon: Some("COLOPHON.md".parse().unwrap()),
      creator: Some("foo".parse().unwrap()),
      description: Some("bar".parse().unwrap()),
      homepage: Some("http://example.com".parse().unwrap()),
      time: Some("2024-01-01".parse().unwrap()),
      title: Some("foo-A0".parse().unwrap()),
    });
  }
}
