use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum TrackInfo {
  #[n(0)]
  Audio,
  #[n(1)]
  Video {
    #[n(0)]
    dimensions: Dimensions,
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
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
      },
      "8201a100a200010102",
    );
  }
}
