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
  pub(crate) fn info(&self, video: &Video, index: usize) -> Info {
    let builder = InfoBuilder::new()
      .value("track", Ordinal(index))
      .value(
        "type",
        match self.info {
          TrackInfo::Audio { .. } => "audio",
          TrackInfo::Video { .. } => "video",
        },
      )
      .value("codec", self.codec);

    let builder = match self.info {
      TrackInfo::Audio {
        channels,
        sample_rate,
      } => builder
        .value("channels", channels)
        .value("sample rate", DisplaySampleRate(sample_rate))
        .optional("bit rate", DisplayBitrate::new(video.duration, self.size)),
      TrackInfo::Video {
        bit_depth,
        chroma_subsampling,
        dimensions,
        frames,
        orientation,
      } => {
        let pixels =
          u128::from(dimensions.width) * u128::from(dimensions.height) * u128::from(frames);

        builder
          .value("dimensions", dimensions)
          .value("orientation", orientation)
          .value("frames", frames)
          .when(video.duration > 0, |builder| {
            builder.value(
              "frame rate",
              DisplayFrameRate {
                duration: video.duration,
                frames,
              },
            )
          })
          .optional("bit rate", DisplayBitrate::new(video.duration, self.size))
          .when(pixels > 0, |builder| {
            builder.value(
              "bits per pixel",
              DisplayBitsPerPixel {
                pixels,
                size: self.size,
              },
            )
          })
          .value("bit depth", format!("{bit_depth}-bit"))
          .value("chroma subsampling", chroma_subsampling)
      }
    };

    builder.value("size", format_size(self.size)).build()
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
          bit_depth: 8,
          chroma_subsampling: ChromaSubsampling::Yuv420,
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
        bit_depth: 8,
        chroma_subsampling: ChromaSubsampling::Yuv420,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 240,
        orientation: Orientation::new(),
      },
      size: 1500,
    };

    let mut video = Video::test("foo.mp4");

    video.duration = 10_000;

    assert_eq!(
      track.info(&video, 0),
      InfoBuilder::new()
        .value("track", "1")
        .value("type", "video")
        .value("codec", "H.264")
        .value("dimensions", "2×1")
        .value("orientation", "0°")
        .value("frames", "240")
        .value("frame rate", "24 fps")
        .value("bit rate", "1.2 kbit/s")
        .value("bits per pixel", "25")
        .value("bit depth", "8-bit")
        .value("chroma subsampling", "4:2:0")
        .value("size", "1.5 KiB")
        .build(),
    );

    video.duration = 0;

    assert_eq!(
      track.info(&video, 0),
      InfoBuilder::new()
        .value("track", "1")
        .value("type", "video")
        .value("codec", "H.264")
        .value("dimensions", "2×1")
        .value("orientation", "0°")
        .value("frames", "240")
        .value("bits per pixel", "25")
        .value("bit depth", "8-bit")
        .value("chroma subsampling", "4:2:0")
        .value("size", "1.5 KiB")
        .build(),
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
      track.info(&video, 0),
      InfoBuilder::new()
        .value("track", "1")
        .value("type", "audio")
        .value("codec", "AAC")
        .value("channels", "2")
        .value("sample rate", "44.1 kHz")
        .value("bit rate", "1 kbit/s")
        .value("size", "1.2 KiB")
        .build(),
    );

    video.duration = 0;

    assert_eq!(
      track.info(&video, 0),
      InfoBuilder::new()
        .value("track", "1")
        .value("type", "audio")
        .value("codec", "AAC")
        .value("channels", "2")
        .value("sample rate", "44.1 kHz")
        .value("size", "1.2 KiB")
        .build(),
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
          bit_depth: 8,
          chroma_subsampling: ChromaSubsampling::Yuv420,
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
  }
}
