use super::*;

pub(crate) struct Count<N> {
  n: N,
  noun: &'static str,
  plural: Option<&'static str>,
}

impl<N> Count<N> {
  pub(crate) fn irregular(n: N, noun: &'static str, plural: &'static str) -> Self {
    Self {
      n,
      noun,
      plural: Some(plural),
    }
  }

  pub(crate) fn new(n: N, noun: &'static str) -> Self {
    Self {
      n,
      noun,
      plural: None,
    }
  }
}

impl<N: Display + One + PartialEq> Display for Count<N> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.n.is_one() {
      write!(f, "{} {}", self.n, self.noun)?;
    } else if let Some(plural) = self.plural {
      write!(f, "{} {plural}", self.n)?;
    } else {
      write!(f, "{} {}s", self.n, self.noun)?;
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
