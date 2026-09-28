use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum TrackInfo {
  #[n(0)]
  Audio {
    #[n(1)]
    channels: u64,
    #[n(2)]
    sample_rate: u64,
  },
  #[n(1)]
  Video {
    #[n(1)]
    bit_depth: u64,
    #[n(2)]
    chroma_subsampling: Option<ChromaSubsampling>,
    #[n(3)]
    dimensions: Dimensions,
    #[n(4)]
    frames: u64,
    #[n(5)]
    orientation: Orientation,
  },
}
