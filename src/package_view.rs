use super::*;

#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PackageView {
  Grid,
  #[default]
  List,
}
