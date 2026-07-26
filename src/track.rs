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
  pub(crate) fn info(&self, video: &Video) -> Info {
    let mut entries = vec![
      (
        "type".into(),
        Info::Value(
          match self.info {
            TrackInfo::Audio { .. } => "audio",
            TrackInfo::Video { .. } => "video",
          }
          .into(),
        ),
      ),
      ("codec".into(), Info::Value(self.codec.to_string())),
    ];

    match self.info {
      TrackInfo::Audio {
        channels,
        sample_rate,
      } => {
        entries.push(("channels".into(), Info::Value(channels.to_string())));
        entries.push((
          "sample rate".into(),
          Info::Value(DisplaySampleRate(sample_rate).to_string()),
        ));
      }
      TrackInfo::Video {
        bit_depth,
        dimensions,
        frames,
        orientation,
      } => {
        entries.push(("dimensions".into(), Info::Value(dimensions.to_string())));
        entries.push(("orientation".into(), Info::Value(orientation.to_string())));
        entries.push(("frames".into(), Info::Value(frames.to_string())));

        if video.duration > 0 {
          entries.push((
            "frame rate".into(),
            Info::Value(
              DisplayFrameRate {
                duration: video.duration,
                frames,
              }
              .to_string(),
            ),
          ));
        }

        if let Some(bit_depth) = bit_depth {
          entries.push(("bit depth".into(), Info::Value(format!("{bit_depth}-bit"))));
        }
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
        info: TrackInfo::Audio {
          channels: 2,
          sample_rate: 44100,
        },
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
        info: TrackInfo::Audio {
          channels: 2,
          sample_rate: 44100,
        },
        size: 0,
      },
      "a30000018200a200020119ac440200",
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
  fn info() {
    let track = Track {
      codec: Codec::H264,
      info: TrackInfo::Video {
        bit_depth: Some(8),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 240,
        orientation: Orientation::new(),
      },
      size: 0,
    };

    let mut video = "foo.mp4".parse::<Video>().unwrap();

    video.duration = 10_000;

    assert_eq!(
      track.info(&video),
      Info::Map(vec![
        ("type".into(), Info::Value("video".into())),
        ("codec".into(), Info::Value("H.264".into())),
        ("dimensions".into(), Info::Value("2×1".into())),
        ("orientation".into(), Info::Value("0°".into())),
        ("frames".into(), Info::Value("240".into())),
        ("frame rate".into(), Info::Value("24 fps".into())),
        ("bit depth".into(), Info::Value("8-bit".into())),
        ("size".into(), Info::Value("0 B".into())),
      ]),
    );

    video.duration = 0;

    assert_eq!(
      track.info(&video),
      Info::Map(vec![
        ("type".into(), Info::Value("video".into())),
        ("codec".into(), Info::Value("H.264".into())),
        ("dimensions".into(), Info::Value("2×1".into())),
        ("orientation".into(), Info::Value("0°".into())),
        ("frames".into(), Info::Value("240".into())),
        ("bit depth".into(), Info::Value("8-bit".into())),
        ("size".into(), Info::Value("0 B".into())),
      ]),
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio {
          channels: 2,
          sample_rate: 44100,
        },
        size: 0,
      })
      .unwrap(),
      r#"{"codec":"aac","info":{"type":"audio","channels":2,"sample_rate":44100},"size":0}"#,
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
