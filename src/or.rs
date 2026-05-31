use super::*;

pub(crate) struct Or {
  items: Vec<String>,
}

impl Or {
  pub(crate) fn new(items: impl IntoIterator<Item = String>) -> Self {
    Self {
      items: items.into_iter().collect(),
    }
  }
}

impl Display for Or {
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

          write!(f, "{}", item)?;
        }

        Ok(())
      }
    }
  }
}
