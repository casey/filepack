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

impl Track {
  pub(crate) fn info(&self) -> Info {
    let mut entries = vec![
      (
        "type".into(),
        Info::Value(
          match self.info {
            TrackInfo::Audio => "audio",
            TrackInfo::Video { .. } => "video",
          }
          .into(),
        ),
      ),
      ("codec".into(), Info::Value(self.codec.to_string())),
    ];

    if let TrackInfo::Video {
      bit_depth,
      dimensions,
      frames,
    } = self.info
    {
      entries.push(("dimensions".into(), Info::Value(dimensions.to_string())));
      entries.push(("frames".into(), Info::Value(frames.to_string())));

      if let Some(bit_depth) = bit_depth {
        entries.push(("bit depth".into(), Info::Value(format!("{bit_depth}-bit"))));
      }
    }

    entries.push((
      "size".into(),
      Info::Value(format_size(self.size).to_string()),
    ));

    Info::Map(entries)
  }
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
          bit_depth: Some(8),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
          orientation: Orientation::new(),
        },
        size: 0,
      },
      "H.264 2×1",
    );

    case(
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          bit_depth: Some(8),
          dimensions: Dimensions {
            height: 1080,
            width: 1920,
          },
          frames: 0,
          orientation: Orientation::new(),
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
          bit_depth: Some(8),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
          orientation: Orientation::new(),
        },
        size: 0,
      },
      "a30001018201a4000801a200010102020003a200f401000200",
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
          bit_depth: Some(8),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
          orientation: Orientation::new(),
        },
        size: 0,
      })
      .unwrap(),
      r#"{"codec":"h264","info":{"type":"video","bit_depth":8,"dimensions":{"height":1,"width":2},"frames":0,"orientation":{"mirrored":false,"rotation":0}},"size":0}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          bit_depth: None,
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
          orientation: Orientation::new(),
        },
        size: 0,
      })
      .unwrap(),
      r#"{"codec":"h264","info":{"type":"video","dimensions":{"height":1,"width":2},"frames":0,"orientation":{"mirrored":false,"rotation":0}},"size":0}"#,
    );
  }
}
