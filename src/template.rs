use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Template {
  pub(crate) title: String,
}
