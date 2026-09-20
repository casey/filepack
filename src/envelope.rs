use super::*;

#[derive(Encode)]
pub(crate) struct Envelope<T> {
  #[n(0)]
  pub(crate) application: Application,
  #[n(1)]
  pub(crate) context: Context,
  #[n(2)]
  pub(crate) message: T,
}
