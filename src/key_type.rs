use super::*;

#[derive(EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum KeyType {
  Private,
  Public,
}

impl KeyType {
  pub(crate) fn extension(self) -> &'static str {
    self.into()
  }
}
