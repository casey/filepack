use super::*;

pub(crate) use self::status::Status;

mod status;

/// Rights statement for a package's content.
#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, Json, PartialEq, Serialize)]
pub(crate) struct Rights {
  /// Reason the content is believed to have the stated status.
  #[n(1)]
  pub(crate) basis: Option<Line>,
  /// License of the content.
  #[n(2)]
  pub(crate) license: Option<Line>,
  /// Links supporting the rights statement.
  #[n(3)]
  pub(crate) links: Option<NonEmptyOrderedSet<CheckedUrl>>,
  /// Notices required to accompany the content.
  #[n(4)]
  pub(crate) notices: Option<NonEmptyOrderedSet<Line>>,
  /// Rights status of the content.
  #[n(5)]
  pub(crate) status: Status,
  /// License files.
  #[n(6)]
  pub(crate) texts: Option<NonEmptyOrderedSet<Text>>,
}

impl Rights {
  pub(crate) fn info(&self, identifier: PackageIdentifier) -> Info {
    InfoBuilder::new()
      .value("status", self.status.display())
      .when_some(self.license.as_ref(), |builder, license| {
        if let Ok(License::Identifier(identifier)) = license.parse() {
          builder.info(
            "license",
            Info::Link(
              Link::new(license)
                .href(format!(
                  "https://spdx.org/licenses/{}.html",
                  identifier.name
                ))
                .title(identifier.full_name),
            ),
          )
        } else {
          builder.value("license", license)
        }
      })
      .when_some(self.texts.as_ref(), |builder, texts| {
        builder.list(
          "texts",
          texts.iter().enumerate().map(|(i, text)| {
            Info::Link(Link::new(&text.path).href(PackageText::License(i).path(identifier)))
          }),
        )
      })
      .when_some(self.notices.as_ref(), |builder, notices| {
        builder.list(
          "notices",
          notices.iter().map(|notice| Info::Value(notice.to_string())),
        )
      })
      .optional("basis", self.basis.as_ref())
      .when_some(self.links.as_ref(), |builder, links| {
        builder.list(
          "links",
          links
            .iter()
            .map(|link| Info::Link(Link::new(link).href(link))),
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
    assert_encoding(Rights {
      basis: Some("foo".parse().unwrap()),
      license: Some("baz".parse().unwrap()),
      links: Some(
        vec!["http://example.com".parse().unwrap()]
          .try_into()
          .unwrap(),
      ),
      notices: Some(vec!["bar".parse().unwrap()].try_into().unwrap()),
      status: Status::Licensed,
      texts: Some(
        vec![Text::new("qux.txt".parse().unwrap()).unwrap()]
          .try_into()
          .unwrap(),
      ),
    });
  }
}
