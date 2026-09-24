use super::*;

#[derive(Clone)]
pub(crate) struct PackageSummary {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Option<Metadata>,
  pub(crate) number: u64,
  pub(crate) totals: Totals,
}
