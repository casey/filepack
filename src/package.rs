use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Package {
  pub(crate) creator: Option<Component>,
  pub(crate) creator_tag: Option<Tag>,
  pub(crate) date: Option<DateTime>,
  pub(crate) description: Option<String>,
  pub(crate) nfo: Option<filename::Nfo>,
}
