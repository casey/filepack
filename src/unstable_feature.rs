use super::*;

#[derive(
  Clone, Copy, Debug, Deserialize, Display, EnumIter, Eq, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum UnstableFeature {
  WebPackages,
}
