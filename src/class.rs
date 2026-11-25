use super::*;

pub(crate) struct Class(pub(crate) Option<&'static str>);

impl Display for Class {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if let Some(class) = self.0 {
      write!(f, " class={class}")?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(Class(None).to_string(), "");
    assert_eq!(Class(Some("foo")).to_string(), " class=foo");
  }
}
