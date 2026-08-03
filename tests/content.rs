use super::*;

pub(crate) trait Content {
  fn content(self) -> Vec<u8>;
}

impl Content for &str {
  fn content(self) -> Vec<u8> {
    unindent(self).into_bytes()
  }
}

impl Content for String {
  fn content(self) -> Vec<u8> {
    self.as_str().content()
  }
}

impl Content for &String {
  fn content(self) -> Vec<u8> {
    self.as_str().content()
  }
}

impl Content for Vec<u8> {
  fn content(self) -> Vec<u8> {
    self
  }
}

impl Content for &Vec<u8> {
  fn content(self) -> Vec<u8> {
    self.clone()
  }
}

impl Content for &[u8] {
  fn content(self) -> Vec<u8> {
    self.to_vec()
  }
}
