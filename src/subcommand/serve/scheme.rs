use super::*;

#[derive(Clone, Copy, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum Scheme {
  Http,
  Https,
}

impl Scheme {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
