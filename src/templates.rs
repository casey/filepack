use super::*;

#[derive(Boilerplate)]
pub struct DirectoryHtml {
  pub directory: Directory,
  pub hash: Hash,
}

impl Page for DirectoryHtml {
  fn title(&self) -> String {
    format!("directory {} · filepack", self.hash)
  }
}

#[derive(Boilerplate)]
pub(crate) struct FilesHtml {
  pub(crate) files: Vec<Hash>,
}

impl Page for FilesHtml {
  fn title(&self) -> String {
    "files · filepack".into()
  }
}

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<ComponentBuf>)>,
}

impl Page for PackagesHtml {
  fn title(&self) -> String {
    "packages · filepack".into()
  }
}

#[derive(Boilerplate)]
pub struct PackageHtml {
  pub fingerprint: Fingerprint,
  pub metadata: Option<Metadata>,
}

impl PackageHtml {
  fn title(&self) -> Option<&Component> {
    self.metadata.as_ref()?.title.as_deref()
  }
}

impl Page for PackageHtml {
  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} · filepack")
    } else {
      format!("{} · filepack", self.fingerprint)
    }
  }
}

#[derive(Boilerplate)]
pub struct PageHtml<T: Page> {
  content: T,
}

pub trait Page: Display {
  fn title(&self) -> String;
}

impl<T: Page> From<T> for PageHtml<T> {
  fn from(page: T) -> PageHtml<T> {
    PageHtml { content: page }
  }
}
