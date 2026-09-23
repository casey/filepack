use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ColorInfo {
  pub(crate) bit_depth: u64,
  pub(crate) chroma_subsampling: ChromaSubsampling,
}
