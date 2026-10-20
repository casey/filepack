use super::*;

/// Feature that may change in backwards incompatible ways.
#[derive(
  Clone,
  Copy,
  Debug,
  Deserialize,
  Display,
  EnumIter,
  Eq,
  IntoStaticStr,
  Json,
  Ord,
  PartialEq,
  PartialOrd,
  Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum UnstableFeature {
  /// Packages with `media: web` metadata.
  WebPackages,
}

impl UnstableFeature {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
