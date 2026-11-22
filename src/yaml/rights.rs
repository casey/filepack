use super::*;

/// Rights statement for a package's content.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rights {
  /// Reason the content is believed to have the stated license.
  #[json(example = "Published in 1929.")]
  pub(crate) basis: Option<Text>,
  /// License of the content.
  #[json(example = "public-domain")]
  pub(crate) license: License,
}

impl From<Rights> for crate::Rights {
  fn from(rights: Rights) -> Self {
    let Rights { basis, license } = rights;

    Self {
      basis,
      license: license.to_string().parse().unwrap(),
    }
  }
}
