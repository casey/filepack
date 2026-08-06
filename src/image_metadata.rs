use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct ImageMetadata {
  pub(crate) dimensions: Dimensions,
  pub(crate) orientation: Orientation,
}
