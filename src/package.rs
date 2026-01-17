use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Package {
  pub(crate) date: Option<DateTime>,
  pub(crate) description: Option<String>,
  pub(crate) nfo: Option<filename::Nfo>,
  pub(crate) packager: Option<Component>,
}
