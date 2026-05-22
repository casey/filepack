use super::*;

#[derive(Boilerplate)]
pub struct DirectoryHtml {
  pub directory: Directory,
  pub hash: Hash,
}

#[derive(Boilerplate)]
pub(crate) struct FilesHtml {
  pub(crate) files: Vec<Hash>,
}

#[derive(Boilerplate)]
pub struct PackageHtml {
  pub fingerprint: Fingerprint,
  pub metadata: Option<Metadata>,
}
