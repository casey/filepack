use super::*;

#[derive(Boilerplate)]
pub(crate) struct IndexHtml {
  pub(crate) archives: Vec<Archive>,
}

#[derive(Boilerplate)]
pub(crate) struct PackageHtml {
  pub(crate) archive: Archive,
}
