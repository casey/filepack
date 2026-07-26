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
    dimensions: Dimensions,
    #[n(2)]
    frames: u64,
    #[n(3)]
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
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
        orientation: Orientation::new(),
      },
      "8201a4000801a200010102020003a200f40100",
    );

    assert_cbor(
      TrackInfo::Video {
        bit_depth: None,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
        orientation: Orientation::new(),
      },
      "8201a301a200010102020003a200f40100",
    );
  }
}
