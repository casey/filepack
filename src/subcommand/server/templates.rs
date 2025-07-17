use super::*;

#[derive(Boilerplate)]
pub(crate) struct IndexHtml {
  pub(crate) archives: Vec<Archive>,
}

#[derive(Boilerplate)]
pub(crate) struct PackageHtml {
  pub(crate) archive: Archive,
}

impl PackageHtml {
  #[allow(clippy::unused_self)]
  fn metadata(&self) -> Option<Metadata> {
    None
  }
}

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: Display>(pub(crate) T);
