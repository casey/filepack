use super::*;

#[skip_serializing_none]
#[derive(Clone, Copy, Debug, Decode, Encode, Json, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum TrackInfo {
  /// Information specific to audio tracks.
  #[n(0)]
  Audio {
    /// Number of audio channels.
    #[n(1)]
    channels: u64,
    /// Sample rate in Hertz.
    #[n(2)]
    sample_rate: u64,
  },
  /// Information specific to video tracks.
  #[n(1)]
  Video {
    /// Bits per color channel.
    #[n(1)]
    bit_depth: u64,
    /// Chroma subsampling, for formats that support it.
    #[n(2)]
    chroma_subsampling: Option<ChromaSubsampling>,
    /// Dimensions in pixels.
    #[n(3)]
    dimensions: Dimensions,
    /// Total number of video frames.
    #[n(4)]
    frames: u64,
    /// Orientation to apply when displaying.
    #[n(5)]
    orientation: Orientation,
  },
}
