use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Encode, Decode, Json, PartialEq, Serialize)]
pub(crate) struct PackageMetadata {
  /// Path to a markdown file containing a colophon describing the package.
  #[n(1)]
  pub(crate) colophon: Option<RelativePath>,
  /// Primary person or group who created the package.
  #[n(2)]
  pub(crate) creator: Option<Text>,
  /// Single-line description of the package.
  #[n(3)]
  pub(crate) description: Option<Text>,
  /// Official homepage of the package.
  #[n(4)]
  pub(crate) homepage: Option<CheckedUrl>,
  /// Time the package was created.
  #[n(5)]
  pub(crate) time: Option<Time>,
  /// Title of the package.
  #[n(6)]
  pub(crate) title: Option<Text>,
}

impl PackageMetadata {
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
    assert_encoding(PackageMetadata {
      colophon: Some("COLOPHON.md".parse().unwrap()),
      creator: Some("foo".parse().unwrap()),
      description: Some("bar".parse().unwrap()),
      homepage: Some("http://example.com".parse().unwrap()),
      time: Some("2024-01-01".parse().unwrap()),
      title: Some("foo-A0".parse().unwrap()),
    });
  }
}
