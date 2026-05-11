use super::*;

#[derive(Decode, Encode)]
pub(crate) enum Context {
  #[n(0)]
  Statement,
}
