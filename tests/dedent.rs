use super::*;

pub(crate) trait Dedent {
  fn dedent(self) -> Vec<u8>;
}

impl Dedent for &str {
  fn dedent(self) -> Vec<u8> {
    unindent(self).into_bytes()
  }
}

impl Dedent for String {
  fn dedent(self) -> Vec<u8> {
    self.as_str().dedent()
  }
}

impl Dedent for &String {
  fn dedent(self) -> Vec<u8> {
    self.as_str().dedent()
  }
}

impl Dedent for Vec<u8> {
  fn dedent(self) -> Vec<u8> {
    self
  }
}

impl Dedent for &Vec<u8> {
  fn dedent(self) -> Vec<u8> {
    self.clone()
  }
}

impl Dedent for &[u8] {
  fn dedent(self) -> Vec<u8> {
    self.to_vec()
  }
}
