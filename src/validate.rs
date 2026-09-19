use super::*;

pub(crate) trait Validate {
  fn validate(&self) -> DecodeResult;
}
