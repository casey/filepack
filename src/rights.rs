use super::*;

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
  pub(crate) license: Text,
  /// Links supporting the basis.
  #[n(4)]
  pub(crate) links: Option<NonEmptyOrderedSet<CheckedUrl>>,
}

impl Rights {
  pub(crate) fn info(&self) -> Info {
    InfoBuilder::new()
      .value("license", &self.license)
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
          links.as_slice().iter().map(|link| Info::Link {
            code: false,
            text: link.to_string(),
            url: link.to_string(),
          }),
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
      license: "unknown".parse().unwrap(),
      links: Some(
        vec!["http://example.com".parse().unwrap()]
          .try_into()
          .unwrap(),
      ),
    });
  }
}
