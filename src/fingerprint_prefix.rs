use super::*;

#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum FingerprintPrefix {
  Directory,
  Entry,
  File,
  Message,
}

impl FingerprintPrefix {
  fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn prefix(self) -> String {
    format!("filepack:{}", self.name())
  }
}
