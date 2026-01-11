use super::*;

#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Context {
  Directory,
  Entry,
  File,
  Message,
}

impl Context {
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for Context {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
