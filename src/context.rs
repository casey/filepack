use super::*;

#[derive(Encode)]
pub(crate) enum Context {
  #[n(0)]
  Statement,
}
