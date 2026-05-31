use super::*;

#[derive(Debug)]
pub struct Dimensions {
  pub(crate) height: u32,
  pub(crate) width: u32,
}

impl Display for Dimensions {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}×{}", self.width, self.height)
  }
}
