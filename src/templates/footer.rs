use super::*;

#[derive(Boilerplate)]
pub(crate) struct FooterHtml<'a> {
  pub(crate) download: String,
  pub(crate) heading: String,
  pub(crate) identifier: PackageIdentifier,
  pub(crate) metadata: &'a Metadata,
}
