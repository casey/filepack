use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Metadata {
  title: String,
}

impl Metadata {
  pub(crate) const FILENAME: &'static str = "metadata.json";
}

impl From<Template> for Metadata {
  fn from(Template { title }: Template) -> Self {
    Self { title }
  }
}
