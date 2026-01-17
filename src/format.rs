use super::*;

#[derive(Clone, Copy, Default, IntoStaticStr, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Format {
  Json,
  #[default]
  JsonPretty,
  Tsv,
}

impl Format {
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for Format {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
