use super::*;

pub(crate) struct Or<T> {
  items: Vec<T>,
}

impl<T: Display> Or<T> {
  pub(crate) fn new(items: impl IntoIterator<Item = T>) -> Self {
    Self {
      items: items.into_iter().collect(),
    }
  }
}

impl<T: Display> Display for Or<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.items.len() {
      0 => Ok(()),
      1 => write!(f, "{}", self.items[0]),
      2 => write!(f, "{} or {}", self.items[0], self.items[1]),
      _ => {
        for (i, item) in self.items.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }

          if i + 1 == self.items.len() {
            write!(f, "or ")?;
          }

          write!(f, "{item}")?;
        }

        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn or() {
    #[track_caller]
    fn case(items: &[&str], expected: &str) {
      assert_eq!(Or::new(items).to_string(), expected);
    }

    case(&[], "");
    case(&["foo"], "foo");
    case(&["foo", "bar"], "foo or bar");
    case(&["foo", "bar", "bob"], "foo, bar, or bob");
    case(&["foo", "bar", "bob", "baz"], "foo, bar, bob, or baz");
  }
}
