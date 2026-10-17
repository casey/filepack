use super::*;

pub(crate) trait HasPage {
  fn has_page(&self) -> bool;
}

impl HasPage for deco::Schema {
  fn has_page(&self) -> bool {
    !matches!(
      self,
      Self::Array { .. }
        | Self::Bytes { .. }
        | Self::Map { .. }
        | Self::NonEmptyOrderedSet { .. }
        | Self::SortedSet { .. }
    )
  }
}

impl HasPage for json::Schema {
  fn has_page(&self) -> bool {
    !matches!(
      self,
      Self::Array { .. } | Self::Map { .. } | Self::Set { .. }
    )
  }
}
