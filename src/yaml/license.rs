use super::*;

/// License of a package's content.
#[derive(Clone, Copy, Debug, Deserialize, Display, Json, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum License {
  /// Used with permission of the rights holder.
  Permission,
  /// Not subject to copyright.
  PublicDomain,
  /// All rights reserved.
  Reserved,
  /// License could not be determined.
  Unknown,
}
