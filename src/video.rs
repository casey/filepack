use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Video {
  #[n(0)]
  pub(crate) duration: u64,
  #[n(1)]
  pub(crate) filename: ComponentBuf,
  #[n(2)]
  pub(crate) tracks: Vec<Track>,
  #[n(3)]
  #[serde(rename = "type")]
  pub(crate) ty: VideoType,
}

impl Video {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  pub(crate) fn formats(videos: &[Video]) -> Vec<VideoType> {
    let mut formats = Vec::new();

    for video in videos {
      if !formats.contains(&video.ty) {
        formats.push(video.ty);
      }
    }

    formats
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    let VideoMetadata { duration, tracks } = match self.ty {
      VideoType::Mp4 => Mp4Decoder::read(&path)?,
      VideoType::Webm => WebmDecoder::read(&path)?,
    };

    self.duration = duration;
    self.tracks = tracks;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }

  pub(crate) fn sum_durations(videos: &[Video]) -> Duration {
    videos.iter().fold(Duration::ZERO, |sum, video| {
      sum.saturating_add(Duration::from_millis(video.duration))
    })
  }
}

impl FromStr for Video {
  type Err = ComponentError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let filename = s.parse::<ComponentBuf>()?;

    let Some(ty) = filename.extension().and_then(VideoType::from_extension) else {
      return Err(ComponentError::Extension {
        extensions: VideoType::EXTENSIONS,
      });
    };

    Ok(Self {
      duration: 0,
      filename,
      tracks: Vec::new(),
      ty,
    })
  }
}

impl Item for Video {
  fn info(&self, url: String) -> Info {
    InfoBuilder::new()
      .link("filename", &self.filename, url)
      .value("type", self.ty)
      .value(
        "duration",
        DisplayDuration(Duration::from_millis(self.duration)),
      )
      .list("tracks", self.tracks.iter().map(|track| track.info(self)))
      .build()
  }

  fn path(&self) -> RelativePath {
    self.as_path()
  }

  fn resource_type(&self) -> ResourceType {
    self.resource_type()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn formats() {
    let foo = "foo.mp4".parse::<Video>().unwrap();
    let bar = "bar.mp4".parse::<Video>().unwrap();
    let baz = "baz.webm".parse::<Video>().unwrap();

    assert_eq!(
      Video::formats(&[foo, bar, baz]),
      [VideoType::Mp4, VideoType::Webm],
    );
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: ComponentError) {
      assert_eq!(s.parse::<Video>().unwrap_err(), expected);
    }

    assert_eq!(
      "foo.mp4".parse::<Video>().unwrap(),
      Video {
        duration: 0,
        filename: "foo.mp4".parse().unwrap(),
        tracks: Vec::new(),
        ty: VideoType::Mp4,
      },
    );

    assert_eq!(
      "foo.webm".parse::<Video>().unwrap(),
      Video {
        duration: 0,
        filename: "foo.webm".parse().unwrap(),
        tracks: Vec::new(),
        ty: VideoType::Webm,
      },
    );

    case(
      "foo.avi",
      ComponentError::Extension {
        extensions: &["mp4", "webm"],
      },
    );
    case(
      "foo",
      ComponentError::Extension {
        extensions: &["mp4", "webm"],
      },
    );
    case("", ComponentError::Empty);
    case("foo/bar.mp4", ComponentError::Separator { character: '/' });
  }

  #[test]
  fn populate() {
    #[track_caller]
    fn case(bytes: &[u8]) -> Result<Video> {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.mp4"), bytes).unwrap();

      let mut video = "foo.mp4".parse::<Video>().unwrap();

      video.populate(&root).map(|()| video)
    }

    assert_eq!(
      case(
        &Mp4Builder::new()
          .duration(2)
          .video_track(2, 1)
          .audio_track(0x40)
          .build()
      )
      .unwrap(),
      Video {
        duration: 2,
        filename: "foo.mp4".parse().unwrap(),
        tracks: vec![
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
          Track {
            codec: Codec::Aac,
            info: TrackInfo::Audio {
              channels: 2,
              sample_rate: 44100,
            },
            size: 0,
          },
        ],
        ty: VideoType::Mp4,
      },
    );

    assert_eq!(
      case(
        &Mp4Builder::new()
          .video_track(2, 1)
          .audio_track(0x6b)
          .build()
      )
      .unwrap()
      .tracks,
      vec![
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
        Track {
          codec: Codec::Mp3,
          info: TrackInfo::Audio {
            channels: 2,
            sample_rate: 44100,
          },
          size: 0,
        },
      ],
    );

    assert_matches_regex!(
      case(b"foo").unwrap_err().to_string(),
      r"^invalid video `.*foo\.mp4`$",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Video {
        duration: 0,
        filename: "foo.mp4".parse().unwrap(),
        tracks: vec![
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
          Track {
            codec: Codec::Mp3,
            info: TrackInfo::Audio {
              channels: 2,
              sample_rate: 44100,
            },
            size: 0,
          },
        ],
        ty: VideoType::Mp4,
      })
      .unwrap(),
      r#"{"duration":0,"filename":"foo.mp4","tracks":[{"codec":"h264","info":{"type":"video","bit_depth":8,"chroma_subsampling":"4:2:0","dimensions":{"height":1,"width":2},"frames":0,"orientation":{"mirrored":false,"rotation":0}},"size":0},{"codec":"mp3","info":{"type":"audio","channels":2,"sample_rate":44100},"size":0}],"type":"mp4"}"#,
    );
  }

  #[test]
  fn sum_durations() {
    #[track_caller]
    fn case(durations: &[u64], expected: Duration) {
      let videos = durations
        .iter()
        .map(|duration| {
          let mut video = "foo.mp4".parse::<Video>().unwrap();
          video.duration = *duration;
          video
        })
        .collect::<Vec<Video>>();

      assert_eq!(Video::sum_durations(&videos), expected);
    }

    case(&[], Duration::ZERO);
    case(&[1500, 600], Duration::from_millis(2100));
  }
}
