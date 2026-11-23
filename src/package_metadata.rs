use super::*;

/// Metadata describing a package itself as opposed to its content.
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
  /// URLs where the package's content was obtained.
  #[n(5)]
  pub(crate) sources: Option<NonEmptyOrderedSet<CheckedUrl>>,
  /// Time the package was created.
  #[n(6)]
  pub(crate) time: Option<Time>,
  /// Title of the package.
  #[n(7)]
  pub(crate) title: Option<Text>,
}

impl PackageMetadata {
  pub(crate) fn info(&self, colophon: Option<Hash>) -> Info {
    InfoBuilder::new()
      .optional("title", self.title.as_ref())
      .optional("creator", self.creator.as_ref())
      .optional("time", self.time.as_ref())
      .optional("description", self.description.as_ref())
      .when_some(self.colophon.as_ref(), |builder, path| {
        builder.link(
          "colophon",
          Link::new("view").href(file_href(colophon.unwrap(), path.filename())),
        )
      })
      .when_some(self.homepage.as_ref(), |builder, homepage| {
        builder.link("homepage", Link::new(homepage).href(homepage))
      })
      .when_some(self.sources.as_ref(), |builder, sources| {
        builder.list(
          "sources",
          sources
            .iter()
            .map(|source| Info::Link(Link::new(source).href(source))),
        )
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
      sources: Some(
        vec!["http://example.com/baz".parse().unwrap()]
          .try_into()
          .unwrap(),
      ),
      time: Some("2024-01-01".parse().unwrap()),
      title: Some("foo-A0".parse().unwrap()),
    });
  }
}
