use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum TrackInfo {
  #[n(0)]
  Audio {
    #[n(0)]
    channels: u64,
    #[n(1)]
    sample_rate: u64,
  },
  #[n(1)]
  Video {
    #[n(0)]
    bit_depth: u64,
    #[n(1)]
    chroma_subsampling: ChromaSubsampling,
    #[n(2)]
    dimensions: Dimensions,
    #[n(3)]
    frames: u64,
    #[n(4)]
    orientation: Orientation,
  },
}
