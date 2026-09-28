use super::*;

#[derive(Debug, PartialEq)]
pub struct Shortcut {
  pub(crate) description: &'static str,
  pub(crate) key: char,
}
