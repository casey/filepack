use super::*;

#[derive(Debug, Encode, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
pub enum Context {
  #[n(0)]
  Claims,
  #[n(1)]
  Statement,
}
