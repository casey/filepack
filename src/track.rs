use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
pub struct Track {
  #[n(0)]
  pub(crate) codec: Codec,
  #[n(1)]
  #[serde(flatten)]
  pub(crate) ty: TrackType,
}

impl Display for Track {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.codec)?;

    if let TrackType::Video { dimensions } = self.ty {
      write!(f, " {dimensions}")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(track: Track, expected: &str) {
      assert_eq!(track.to_string(), expected);
    }

    case(
      Track {
        codec: Codec::Aac,
        ty: TrackType::Audio,
      },
      "AAC",
    );

    case(
      Track {
        codec: Codec::H264,
        ty: TrackType::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
        },
      },
      "H264 2×1",
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      Track {
        codec: Codec::Aac,
        ty: TrackType::Audio,
      },
      "a200000100",
    );

    assert_cbor(
      Track {
        codec: Codec::H264,
        ty: TrackType::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
        },
      },
      "a20001018201a100a200010102",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::Aac,
        ty: TrackType::Audio,
      })
      .unwrap(),
      r#"{"codec":"aac","type":"audio"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::H264,
        ty: TrackType::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
        },
      })
      .unwrap(),
      r#"{"codec":"h264","type":"video","dimensions":{"height":1,"width":2}}"#,
    );
  }
}
