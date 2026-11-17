use super::*;

/// Message type enclosed by an envelope.
#[expect(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Encode, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
pub enum Context {
  /// Revision hash.
  #[n(0)]
  Revision,
  /// Claims in an authorization token.
  #[n(1)]
  Claims,
}
