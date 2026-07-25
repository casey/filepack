use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Value {
  Group(Vec<(String, Value)>),
  Scalar(String),
}

impl Value {
  pub(crate) fn scalar(value: impl ToString) -> Self {
    Self::Scalar(value.to_string())
  }
}
