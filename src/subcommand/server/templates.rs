use super::*;

#[derive(Boilerplate)]
pub(crate) struct IndexHtml {
  pub(crate) packages: Vec<Package>,
}

impl PageContent for IndexHtml {
  fn title(&self) -> String {
    "filepack server".into()
  }
}

#[derive(Boilerplate)]
pub(crate) struct PackageHtml {
  pub(crate) package: Package,
}

impl PageContent for PackageHtml {
  fn title(&self) -> String {
    format!("filepack package {}", self.package.fingerprint)
  }
}

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: PageContent>(pub(crate) T);

pub trait PageContent: Display + 'static {
  fn page(self) -> PageHtml<Self>
  where
    Self: Sized,
  {
    PageHtml(self)
  }

  fn title(&self) -> String;
}
