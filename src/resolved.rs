use super::*;

pub(crate) struct Resolved {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) number: Option<u64>,
  pub(crate) revision: Option<Revision>,
}
