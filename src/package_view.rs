use super::*;

#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageView {
  Details,
  #[default]
  List,
}
