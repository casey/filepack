use super::*;

#[derive(Debug, Encode, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
pub enum Context {
  /// Claims in an authorization token.
  #[n(0)]
  Claims,
  /// Statement attesting to a package.
  #[n(1)]
  Statement,
}
