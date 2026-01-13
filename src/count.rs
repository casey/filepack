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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn count() {
    assert_eq!(Count(0, "foo").to_string(), "0 foos");
    assert_eq!(Count(1, "foo").to_string(), "1 foo");
    assert_eq!(Count(2, "foo").to_string(), "2 foos");
  }
}
