use super::*;

#[derive(Debug)]
pub struct Ticked(BTreeSet<String>);

impl Display for Ticked {
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

impl From<BTreeSet<String>> for Ticked {
  fn from(value: BTreeSet<String>) -> Self {
    Self(value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(Ticked(["foo".into()].into()).to_string(), "`foo`");
    assert_eq!(
      Ticked(["bar".into(), "foo".into()].into()).to_string(),
      "`bar`, `foo`"
    );
  }
}
