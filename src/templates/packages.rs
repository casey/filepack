use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<ComponentBuf>)>,
}

impl Page for PackagesHtml {
  fn title(&self) -> String {
    "packages · filepack".into()
  }
}
