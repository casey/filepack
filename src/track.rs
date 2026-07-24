use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
pub(crate) struct Track {
  #[n(0)]
  pub(crate) codec: Codec,
  #[n(1)]
  pub(crate) info: TrackInfo,
  #[n(2)]
  pub(crate) size: u64,
}

impl Display for Track {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.codec)?;

    if let TrackInfo::Video { dimensions, .. } = self.info {
      if let Some(shorthand) = dimensions.shorthand() {
        write!(f, " {shorthand}")?;
      } else {
        write!(f, " {dimensions}")?;
      }
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
        info: TrackInfo::Audio,
        size: 0,
      },
      "AAC",
    );

    case(
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
        },
        size: 0,
      },
      "H.264 2×1",
    );

    case(
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1080,
            width: 1920,
          },
          frames: 0,
        },
        size: 0,
      },
      "H.264 1080p",
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio,
        size: 0,
      },
      "a3000001000200",
    );

    assert_cbor(
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
        },
        size: 0,
      },
      "a30001018201a200a20001010201000200",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio,
        size: 0,
      })
      .unwrap(),
      r#"{"codec":"aac","info":{"type":"audio"},"size":0}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
        },
        size: 0,
      })
      .unwrap(),
      r#"{"codec":"h264","info":{"type":"video","dimensions":{"height":1,"width":2},"frames":0},"size":0}"#,
    );
  }
}
