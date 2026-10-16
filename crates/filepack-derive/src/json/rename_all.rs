use super::*;

#[derive(Clone, Copy, EnumString)]
pub(crate) enum RenameAll {
  #[strum(serialize = "snake_case")]
  SnakeCase,
}

impl RenameAll {
  pub(crate) fn apply(self, name: &str) -> String {
    match self {
      Self::SnakeCase => name.to_snake_case(),
    }
  }
}
