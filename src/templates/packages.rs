use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<Metadata>)>,
}

impl PackagesHtml {
  fn packages(
    &self,
  ) -> impl Iterator<Item = (Fingerprint, Option<&ComponentBuf>, Option<&ComponentBuf>)> {
    self.packages.iter().map(|(fingerprint, metadata)| {
      (
        *fingerprint,
        metadata
          .as_ref()
          .and_then(|metadata| metadata.creator.as_ref()),
        metadata
          .as_ref()
          .and_then(|metadata| metadata.title.as_ref()),
      )
    })
  }
}

impl Page for PackagesHtml {
  fn title(&self) -> String {
    "packages · filepack".into()
  }
}
