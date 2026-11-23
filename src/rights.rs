use super::*;

pub(crate) use self::status::Status;

mod status;

/// Rights statement for a package's content.
#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, Json, PartialEq, Serialize)]
pub(crate) struct Rights {
  /// Reason the content is believed to have the stated license.
  #[n(1)]
  pub(crate) basis: Option<Text>,
  /// Credit lines to display with the content.
  #[n(2)]
  pub(crate) credits: Option<NonEmptyOrderedSet<Text>>,
  /// License of the content.
  #[n(3)]
  pub(crate) license: Option<Text>,
  /// Links supporting the basis.
  #[n(4)]
  pub(crate) links: Option<NonEmptyOrderedSet<CheckedUrl>>,
  /// Rights status of the content.
  #[n(5)]
  pub(crate) status: Status,
}

impl Rights {
  pub(crate) fn info(&self) -> Info {
    InfoBuilder::new()
      .value("status", self.status)
      .when_some(self.license.as_ref(), |builder, license| {
        if let Ok(License(identifier)) = license.parse() {
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
      .when_some(self.credits.as_ref(), |builder, credits| {
        builder.list(
          "credits",
          credits
            .as_slice()
            .iter()
            .map(|credit| Info::Value(credit.to_string())),
        )
      })
      .optional("basis", self.basis.as_ref())
      .when_some(self.links.as_ref(), |builder, links| {
        builder.list(
          "links",
          links
            .as_slice()
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
      credits: Some(vec!["bar".parse().unwrap()].try_into().unwrap()),
      license: Some("baz".parse().unwrap()),
      links: Some(
        vec!["http://example.com".parse().unwrap()]
          .try_into()
          .unwrap(),
      ),
      status: Status::Licensed,
    });
  }
}
