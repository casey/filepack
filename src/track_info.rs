use super::*;

#[skip_serializing_none]
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum TrackInfo {
  #[n(0)]
  Audio,
  #[n(1)]
  Video {
    #[n(0)]
    bit_depth: Option<u64>,
    #[n(1)]
    dimensions: Dimensions,
    #[n(2)]
    frames: u64,
  },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(TrackInfo::Audio, "00");

    assert_cbor(
      TrackInfo::Video {
        bit_depth: Some(8),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
      },
      "8201a3000801a2000101020200",
    );

    assert_cbor(
      TrackInfo::Video {
        bit_depth: None,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
      },
      "8201a201a2000101020200",
    );
  }
}
