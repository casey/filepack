use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Encode)]
pub(crate) struct Envelope<T> {
  #[n(0)]
  pub(crate) version: Version,
  #[n(1)]
  pub(crate) application: Application,
  #[n(2)]
  pub(crate) context: Context,
  #[n(3)]
  pub(crate) message: T,
}
