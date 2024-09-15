use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Metadata {
  title: String,
}

impl From<Template> for Metadata {
  fn from(Template { title }: Template) -> Self {
    Self { title }
  }
}
