use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<Metadata>)>,
}

impl PackagesHtml {
  fn packages(
    &self,
  ) -> impl Iterator<Item = (Fingerprint, Option<&Component>, Option<&Component>)> {
    self.packages.iter().map(|(fingerprint, metadata)| {
      (
        *fingerprint,
        metadata
          .as_ref()
          .and_then(|metadata| metadata.creator.as_deref()),
        metadata
          .as_ref()
          .and_then(|metadata| metadata.title.as_deref()),
      )
    })
  }
}

impl Page for PackagesHtml {
  fn title(&self) -> String {
    "packages · filepack".into()
  }
}
