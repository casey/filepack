use super::*;

pub(crate) struct Row<T>(pub(crate) T);

impl<T> Row<T> {
  fn write(f: &mut Formatter, cells: &[&dyn Display]) -> fmt::Result {
    write!(f, "|")?;

    for cell in cells {
      write!(f, " {} |", cell.to_string().replace('\n', " "))?;
    }

    Ok(())
  }
}

impl<A: Display, B: Display> Display for Row<(A, B)> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let (a, b) = &self.0;
    Self::write(f, &[a, b])
  }
}

impl<A: Display, B: Display, C: Display> Display for Row<(A, B, C)> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let (a, b, c) = &self.0;
    Self::write(f, &[a, b, c])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(Row(("foo", "bar\nbaz")).to_string(), "| foo | bar baz |");
  }
}
