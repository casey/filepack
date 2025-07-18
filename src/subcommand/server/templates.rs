use super::*;

#[derive(Boilerplate)]
pub(crate) struct IndexHtml {
  pub(crate) archives: Vec<Archive>,
}

impl PageContent for IndexHtml {
  fn title(&self) -> String {
    "filepack server".into()
  }
}

#[derive(Boilerplate)]
pub(crate) struct PackageHtml {
  pub(crate) archive: Archive,
}

impl PageContent for PackageHtml {
  fn title(&self) -> String {
    format!("filepack package {}", self.archive.hash)
  }
}

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: PageContent>(pub(crate) T);

pub trait PageContent: Display + 'static {
  fn title(&self) -> String;

  fn page(self) -> PageHtml<Self>
  where
    Self: Sized,
  {
    PageHtml(self)
  }
}
