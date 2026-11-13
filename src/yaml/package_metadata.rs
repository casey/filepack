use super::*;

/// Metadata describing the package itself, as opposed to its content.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct PackageMetadata {
  /// Path to a markdown file containing a colophon describing the package.
  #[json(example = "COLOPHON.md")]
  pub(crate) colophon: Option<RelativePath>,
  /// Primary person or group who created the package.
  #[json(example = "Egon Spengler")]
  pub(crate) creator: Option<Text>,
  /// Single-line description of the package.
  #[json(
    example = "First edition on loan from NYPL Main Branch research stacks. Captured via Microtek \
               MS-300A flatbed scanner."
  )]
  pub(crate) description: Option<Text>,
  /// Official homepage of the package.
  #[json(example = "https://ghost-busters.net/~egon")]
  pub(crate) homepage: Option<CheckedUrl>,
  /// Time the package was created.
  #[json(example = "1984-07-08")]
  pub(crate) time: Option<Time>,
  /// Title of the package.
  #[json(example = "Tobin's Spirit Guide - First Edition")]
  pub(crate) title: Option<Text>,
}

impl From<PackageMetadata> for crate::PackageMetadata {
  fn from(package: PackageMetadata) -> Self {
    let PackageMetadata {
      colophon,
      creator,
      description,
      homepage,
      time,
      title,
    } = package;

    Self {
      colophon,
      creator,
      description,
      homepage,
      time,
      title,
    }
  }
}
