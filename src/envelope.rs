use super::*;

#[derive(Encode)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) struct Envelope {
  #[n(0)]
  pub(crate) application: &'static str,
  #[n(1)]
  pub(crate) ty: &'static str,
  #[n(2)]
  pub(crate) message: Message,
}
