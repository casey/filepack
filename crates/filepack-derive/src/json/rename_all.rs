use super::*;

#[derive(Clone, Copy, EnumString)]
pub(crate) enum RenameAll {
  #[strum(serialize = "kebab-case")]
  KebabCase,
  #[strum(serialize = "snake_case")]
  SnakeCase,
}

impl RenameAll {
  pub(crate) fn apply(self, name: &str) -> String {
    match self {
      Self::KebabCase => name.to_kebab_case(),
      Self::SnakeCase => name.to_snake_case(),
    }
  }
}
