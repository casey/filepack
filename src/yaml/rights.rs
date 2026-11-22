use super::*;

/// Rights statement for a package's content.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rights {
  /// Reason the content is believed to have the stated license.
  #[json(example = "Published in 1929.")]
  pub(crate) basis: Option<Text>,
  /// Credit lines to display with the content.
  #[json(example = "Courtesy of the New York Public Library")]
  pub(crate) credits: Option<NonEmptyOrderedSet<Text>>,
  /// License of the content.
  #[json(example = "public-domain")]
  pub(crate) license: License,
  /// Links supporting the basis.
  #[json(example = "https://tobin-society.org/spirit-guide/copyright")]
  pub(crate) links: Option<NonEmptyOrderedSet<CheckedUrl>>,
}

impl From<Rights> for crate::Rights {
  fn from(rights: Rights) -> Self {
    let Rights {
      basis,
      credits,
      license,
      links,
    } = rights;

    Self {
      basis,
      credits,
      license: license.to_string().parse().unwrap(),
      links,
    }
  }
}
