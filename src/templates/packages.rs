use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<Metadata>)>,
}

impl Page for PackagesHtml {
  fn title(&self) -> String {
    "packages · filepack".into()
  }
}
