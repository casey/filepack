use super::*;

#[derive(Debug)]
pub struct Ticked<T>(pub BTreeSet<T>);

impl<T: Display> Display for Ticked<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (i, value) in self.0.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "`{value}`")?;
    }
    Ok(())
  }
}

impl<T> From<BTreeSet<T>> for Ticked<T> {
  fn from(value: BTreeSet<T>) -> Self {
    Self(value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(Ticked::<&str>(["foo"].into()).to_string(), "`foo`");
    assert_eq!(
      Ticked::<&str>(["bar", "foo"].into()).to_string(),
      "`bar`, `foo`"
    );
  }
}
