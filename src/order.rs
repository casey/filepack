use super::*;

/// Package sort order.
#[derive(Clone, Copy, Default, Deserialize, Display, EnumIter, Eq, Json, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Order {
  /// Smallest first.
  #[default]
  Ascending,
  /// Largest first.
  Descending,
}

impl Order {
  pub(crate) fn toggle(self) -> Self {
    match self {
      Self::Ascending => Self::Descending,
      Self::Descending => Self::Ascending,
    }
  }
}
