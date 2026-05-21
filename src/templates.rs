use super::*;

#[derive(Boilerplate)]
pub(crate) struct DirectoryHtml {
  pub(crate) directory: Directory,
  pub(crate) hash: Hash,
}

#[derive(Boilerplate)]
pub(crate) struct FilesHtml {
  pub(crate) files: Vec<Hash>,
}
