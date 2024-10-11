use super::*;

#[derive(Boilerplate)]
#[boilerplate(filename = "page.html")]
pub(crate) struct Page {
  pub(crate) manifest: Manifest,
  pub(crate) metadata: Option<Metadata>,
}
