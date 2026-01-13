use super::*;

pub(crate) struct Count<N, T>(pub(crate) N, pub(crate) T);

impl<N: Display + One + PartialEq, T: Display> Display for Count<N, T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {}", self.0, self.1)?;

    if !self.0.is_one() {
      write!(f, "s")?;
    }

    Ok(())
  }
}
