use super::*;

#[derive(Encode)]
pub(crate) struct Envelope<T> {
  #[n(1)]
  pub(crate) application: Application,
  #[n(2)]
  pub(crate) context: Context,
  #[n(3)]
  pub(crate) message: T,
}
