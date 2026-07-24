use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
pub(crate) struct Track {
  #[n(0)]
  pub(crate) codec: Codec,
  #[n(1)]
  pub(crate) info: TrackInfo,
}

impl Display for Track {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.codec)?;

    if let TrackInfo::Video { dimensions } = self.info {
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
        },
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
        },
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
      },
      "a200000100",
    );

    assert_cbor(
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
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
        info: TrackInfo::Audio,
      })
      .unwrap(),
      r#"{"codec":"aac","info":{"type":"audio"}}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
        },
      })
      .unwrap(),
      r#"{"codec":"h264","info":{"type":"video","dimensions":{"height":1,"width":2}}}"#,
    );
  }
}
