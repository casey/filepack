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

        if video.duration > 0 {
          entries.push((
            "bit rate".into(),
            Info::Value(
              DisplayBitrate {
                duration: video.duration,
                size: self.size,
              }
              .to_string(),
            ),
          ));
        }
      }
      TrackInfo::Video {
        bit_depth,
        chroma_subsampling,
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

          entries.push((
            "bit rate".into(),
            Info::Value(
              DisplayBitrate {
                duration: video.duration,
                size: self.size,
              }
              .to_string(),
            ),
          ));
        }

        let pixels =
          u128::from(dimensions.width) * u128::from(dimensions.height) * u128::from(frames);

        if pixels > 0 {
          entries.push((
            "bits per pixel".into(),
            Info::Value(
              DisplayBitsPerPixel {
                pixels,
                size: self.size,
              }
              .to_string(),
            ),
          ));
        }

        if let Some(bit_depth) = bit_depth {
          entries.push(("bit depth".into(), Info::Value(format!("{bit_depth}-bit"))));
        }

        if let Some(chroma_subsampling) = chroma_subsampling {
          entries.push((
            "chroma subsampling".into(),
            Info::Value(chroma_subsampling.to_string()),
          ));
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
          chroma_subsampling: Some(ChromaSubsampling::Yuv420),
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
  }

  #[test]
  fn info() {
    let track = Track {
      codec: Codec::H264,
      info: TrackInfo::Video {
        bit_depth: Some(8),
        chroma_subsampling: None,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 240,
        orientation: Orientation::new(),
      },
      size: 1500,
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
        ("bit rate".into(), Info::Value("1.2 kbit/s".into())),
        ("bits per pixel".into(), Info::Value("25".into())),
        ("bit depth".into(), Info::Value("8-bit".into())),
        ("size".into(), Info::Value("1.5 KiB".into())),
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
        ("bits per pixel".into(), Info::Value("25".into())),
        ("bit depth".into(), Info::Value("8-bit".into())),
        ("size".into(), Info::Value("1.5 KiB".into())),
      ]),
    );

    let track = Track {
      codec: Codec::Aac,
      info: TrackInfo::Audio {
        channels: 2,
        sample_rate: 44100,
      },
      size: 1250,
    };

    video.duration = 10_000;

    assert_eq!(
      track.info(&video),
      Info::Map(vec![
        ("type".into(), Info::Value("audio".into())),
        ("codec".into(), Info::Value("AAC".into())),
        ("channels".into(), Info::Value("2".into())),
        ("sample rate".into(), Info::Value("44.1 kHz".into())),
        ("bit rate".into(), Info::Value("1 kbit/s".into())),
        ("size".into(), Info::Value("1.2 KiB".into())),
      ]),
    );

    video.duration = 0;

    assert_eq!(
      track.info(&video),
      Info::Map(vec![
        ("type".into(), Info::Value("audio".into())),
        ("codec".into(), Info::Value("AAC".into())),
        ("channels".into(), Info::Value("2".into())),
        ("sample rate".into(), Info::Value("44.1 kHz".into())),
        ("size".into(), Info::Value("1.2 KiB".into())),
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
          chroma_subsampling: Some(ChromaSubsampling::Yuv420),
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
      r#"{"codec":"h264","info":{"type":"video","bit_depth":8,"chroma_subsampling":"4:2:0","dimensions":{"height":1,"width":2},"frames":0,"orientation":{"mirrored":false,"rotation":0}},"size":0}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          bit_depth: None,
          chroma_subsampling: None,
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
