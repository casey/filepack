use super::*;

#[derive(Boilerplate)]
pub(crate) struct FilesHtml {
  pub(crate) files: Vec<Hash>,
}

impl Page for FilesHtml {
  fn title(&self) -> String {
    "files · filepack".into()
  }
}
