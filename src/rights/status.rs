use super::*;

/// Rights status of a package's content.
#[derive(Clone, Copy, Debug, Decode, Deserialize, Display, Encode, Json, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Status {
  /// Available under a license.
  #[n(0)]
  Licensed,
  /// Distributed with the permission of the rights holder.
  #[n(1)]
  Permission,
  /// Not under copyright.
  #[n(2)]
  PublicDomain,
  /// Under copyright with no rights granted.
  #[n(3)]
  Reserved,
  /// Not known.
  #[n(4)]
  Unknown,
}

impl Status {
  pub(crate) fn display(self) -> String {
    self.to_string().replace('-', " ")
  }
}
