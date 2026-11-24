pub trait OptionExt<T> {
  fn replace_none(&mut self, value: T);
}

impl<T> OptionExt<T> for Option<T> {
  #[track_caller]
  fn replace_none(&mut self, value: T) {
    assert!(self.is_none());
    *self = Some(value);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn replace_none() {
    let mut option = None;
    option.replace_none(0);
    assert_eq!(option, Some(0));
  }

  #[test]
  #[should_panic(expected = "assertion failed: self.is_none()")]
  fn replace_none_some() {
    Some(0).replace_none(1);
  }
}
