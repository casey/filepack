use super::*;

#[derive(Encode)]
pub(crate) struct Envelope {
  #[n(0)]
  pub(crate) application: &'static str,
  #[n(1)]
  pub(crate) ty: &'static str,
  #[n(2)]
  pub(crate) message: Message,
}
