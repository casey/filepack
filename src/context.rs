use super::*;

/// Message type enclosed by an envelope.
#[expect(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Encode, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
pub enum Context {
  /// Statement attesting to a package.
  #[n(0)]
  Statement,
  /// Claims in an authorization token.
  #[n(1)]
  Claims,
}
