use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Metadata {
  pub(crate) title: String,
}

impl Metadata {
  pub(crate) const FILENAME: &'static str = "metadata.json";

  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    serde_json::from_str(&filesystem::read_to_string(path)?)
      .context(error::DeserializeMetadata { path })
  }

  pub(crate) fn to_json(&self) -> String {
    serde_json::to_string(self).unwrap()
  }
}

impl From<Template> for Metadata {
  fn from(Template { title }: Template) -> Self {
    Self { title }
  }
}
