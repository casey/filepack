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

impl Page for PackageHtml {
  fn title(&self) -> String {
    format!("package {} · filepack", self.fingerprint)
  }
}

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: Display + Page> {
  content: T,
}

pub(crate) trait Page {
  fn title(&self) -> String;
}

impl<T: Display + Page> From<T> for PageHtml<T> {
  fn from(page: T) -> PageHtml<T> {
    PageHtml { content: page }
  }
}
