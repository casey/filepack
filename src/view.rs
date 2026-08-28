use super::*;

#[derive(Clone, Copy, Default, Deserialize, Display, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum View {
  Grid,
  #[default]
  List,
}
