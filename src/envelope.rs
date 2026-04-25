use super::*;

#[derive(Encode)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) struct Envelope {
  #[n(0)]
  pub(crate) application: Application,
  #[n(1)]
  pub(crate) context: Context,
  #[n(2)]
  pub(crate) message: Message,
}
