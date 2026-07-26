use super::*;

#[skip_serializing_none]
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
    bit_depth: Option<u64>,
    #[n(1)]
    chroma_subsampling: Option<ChromaSubsampling>,
    #[n(2)]
    dimensions: Dimensions,
    #[n(3)]
    frames: u64,
    #[n(4)]
    orientation: Orientation,
  },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(
      TrackInfo::Audio {
        channels: 2,
        sample_rate: 44100,
      },
      "8200a200020119ac44",
    );

    assert_cbor(
      TrackInfo::Video {
        bit_depth: Some(8),
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
        orientation: Orientation::new(),
      },
      "8201a50008010102a200010102030004a200f40100",
    );

    assert_cbor(
      TrackInfo::Video {
        bit_depth: None,
        chroma_subsampling: None,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
        orientation: Orientation::new(),
      },
      "8201a302a200010102030004a200f40100",
    );
  }
}
