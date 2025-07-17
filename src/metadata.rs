use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Metadata {
  pub(crate) title: String,
}

impl Metadata {
  pub(crate) const FILENAME: &'static str = "metadata.json";

  pub(crate) fn store(&self, path: &Utf8Path) -> Result<()> {
    filesystem::write(path, format!("{}\n", serde_json::to_string(self).unwrap()))
  }
}

impl From<Template> for Metadata {
  fn from(Template { title }: Template) -> Self {
    Self { title }
  }
}
