use super::*;

pub(crate) struct Index(pub(crate) usize);

impl Display for Index {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0.checked_add(1).unwrap())
  }
}
