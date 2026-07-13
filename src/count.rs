use super::*;

pub(crate) struct Count<N>(N, &'static str, Option<&'static str>);

impl<N> Count<N> {
  pub(crate) fn new(n: N, s: &'static str) -> Self {
    Self(n, s, None)
  }

  pub(crate) fn irregular(n: N, s: &'static str, i: &'static str) -> Self {
    Self(n, s, Some(i))
  }
}

impl<N: Display + One + PartialEq> Display for Count<N> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.0.is_one() {
      write!(f, "{} {}", self.0, self.1)?;
    } else {
      if let Some(s) = self.2 {
        write!(f, "{} {s}", self.0)?;
      } else {
        write!(f, "{} {}s", self.0, self.1)?;
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn count() {
    assert_eq!(Count::new(0, "foo").to_string(), "0 foos");
    assert_eq!(Count::new(1, "foo").to_string(), "1 foo");
    assert_eq!(Count::new(2, "foo").to_string(), "2 foos");
    assert_eq!(Count::irregular(0, "bar", "bör").to_string(), "0 bör");
    assert_eq!(Count::irregular(1, "bar", "bör").to_string(), "1 bar");
    assert_eq!(Count::irregular(2, "bar", "bör").to_string(), "2 bör");
  }
}
