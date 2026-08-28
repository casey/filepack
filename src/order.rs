use super::*;

#[derive(Clone, Copy, Default, Deserialize, Display, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Order {
  #[default]
  Ascending,
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
