use super::*;

#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum HashContext {
  Directory,
  Entry,
  File,
  Message,
}

impl HashContext {
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for HashContext {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
