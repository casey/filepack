use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct ImageMetadata {
  pub(crate) alpha: bool,
  pub(crate) bit_depth: u64,
  pub(crate) chroma_subsampling: Option<ChromaSubsampling>,
  pub(crate) color_type: ColorType,
  pub(crate) dimensions: Dimensions,
  pub(crate) orientation: Orientation,
}
