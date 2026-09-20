use super::*;

#[derive(Clone, Copy, Debug, Encode, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
pub enum Context {
  #[n(0)]
  Claims,
  #[n(1)]
  Statement,
}
