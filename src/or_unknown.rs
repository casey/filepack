use super::*;

pub(crate) struct OrUnknown<T>(pub(crate) Option<T>);

impl<T: Display> Display for OrUnknown<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match &self.0 {
      Some(value) => write!(f, "{value}"),
      None => write!(f, "unknown"),
    }
  }
}
