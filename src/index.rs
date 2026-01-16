use super::*;

#[derive(Debug)]
pub struct Index(pub(crate) usize);

impl From<usize> for Index {
  fn from(value: usize) -> Self {
    Self(value)
  }
}

impl Display for Index {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0.checked_add(1).unwrap())
  }
}
