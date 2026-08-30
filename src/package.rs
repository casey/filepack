use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Encode, Decode, PartialEq, Serialize)]
pub(crate) struct Package {
  #[n(0)]
  pub(crate) colophon: Option<RelativePath>,
  #[n(1)]
  pub(crate) creator: Option<Text>,
  #[n(2)]
  pub(crate) description: Option<Text>,
  #[n(3)]
  pub(crate) homepage: Option<CheckedUrl>,
  #[n(4)]
  pub(crate) time: Option<Time>,
  #[n(5)]
  pub(crate) title: Option<Text>,
}

impl Package {
  pub(crate) fn info(&self, colophon: Option<Hash>) -> Info {
    InfoBuilder::new()
      .optional("title", self.title.as_ref())
      .optional("creator", self.creator.as_ref())
      .optional("time", self.time.as_ref())
      .optional("description", self.description.as_ref())
      .when_some(
        self.colophon.as_ref().zip(colophon),
        |builder, (path, hash)| {
          builder.link(
            "colophon",
            "view",
            format!("/file/{hash}/{}", path.percent_encode_path()),
          )
        },
      )
      .when_some(self.homepage.as_ref(), |builder, homepage| {
        builder.link("homepage", homepage, homepage.to_string())
      })
      .build()
  }
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
