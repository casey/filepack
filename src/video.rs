use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, PartialEq, Serialize)]
pub(crate) struct Video {
  #[n(0)]
  pub(crate) duration: u64,
  #[n(1)]
  pub(crate) path: RelativePath,
  #[n(2)]
  pub(crate) placeholder: Option<Image>,
  #[n(3)]
  pub(crate) tracks: Vec<Track>,
  #[n(4)]
  #[serde(rename = "type")]
  pub(crate) ty: VideoType,
}

impl Video {
  pub(crate) fn oriented_dimensions(&self) -> Option<Dimensions> {
    self.tracks.iter().find_map(|track| match track.info {
      TrackInfo::Video {
        dimensions,
        orientation,
        ..
      } => Some(orientation.dimensions(dimensions)),
      TrackInfo::Audio { .. } => None,
    })
  }

  pub(crate) fn sum_durations(videos: &[Item<Video>]) -> Duration {
    videos.iter().fold(Duration::ZERO, |sum, video| {
      sum.saturating_add(Duration::from_millis(video.content.duration))
    })
  }
}

impl Content for Video {
  const LABEL: &'static str = "Video";

  type Type = VideoType;

  fn info(&self, builder: InfoBuilder) -> InfoBuilder {
    builder
      .value("type", self.ty)
      .value(
        "duration",
        DisplayDuration(Duration::from_millis(self.duration)),
      )
      .list("tracks", self.tracks.iter().map(|track| track.info(self)))
  }

  fn load(root: &Utf8Path, path: RelativePath) -> Result<Item<Self>> {
    let ty = VideoType::from_path(&path).context(error::Path { path: &path })?;

    let VideoMetadata {
      duration,
      title,
      tracks,
    } = match ty {
      VideoType::Mp4 => Mp4Decoder::read(&root.join(&path))?,
      VideoType::Webm => WebmDecoder::read(&root.join(&path))?,
    };

    Ok(Item {
      content: Self {
        duration,
        path,
        placeholder: None,
        tracks,
        ty,
      },
      title,
    })
  }

  fn path(&self) -> &RelativePath {
    &self.path
  }

  fn placeholder(&self) -> Option<&Image> {
    self.placeholder.as_ref()
  }

  #[cfg(test)]
  fn test(path: &str) -> Self {
    let path = path.parse::<RelativePath>().unwrap();
    let ty = VideoType::from_path(&path).unwrap();
    Self {
      duration: 1000,
      path,
      placeholder: None,
      tracks: Vec::new(),
      ty,
    }
  }

  fn ty(&self) -> Self::Type {
    self.ty
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Video {
      placeholder: Some(Image::test("bar.png")),
      ..Video::test("foo.mp4")
    });
  }

  #[test]
  fn load() {
    #[track_caller]
    fn case(bytes: &[u8]) -> Result<Video> {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.mp4"), bytes).unwrap();

      Video::load(&root, "foo.mp4".parse().unwrap()).map(|item| item.content)
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
        path: "foo.mp4".parse().unwrap(),
        placeholder: None,
        tracks: vec![
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
  fn oriented_dimensions() {
    #[track_caller]
    fn case(tracks: Vec<Track>, expected: Option<Dimensions>) {
      let video = Video {
        tracks,
        ..Video::test("foo.mp4")
      };

      assert_eq!(video.oriented_dimensions(), expected);
    }

    fn track(rotation: Rotation) -> Track {
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
          orientation: Orientation {
            mirrored: false,
            rotation,
          },
        },
        size: 0,
      }
    }

    case(Vec::new(), None);
    case(
      vec![track(Rotation::R0)],
      Some(Dimensions {
        height: 1,
        width: 2,
      }),
    );
    case(
      vec![track(Rotation::R90)],
      Some(Dimensions {
        height: 2,
        width: 1,
      }),
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Video {
        duration: 0,
        path: "foo.mp4".parse().unwrap(),
        placeholder: None,
        tracks: vec![
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
      r#"{"duration":0,"path":"foo.mp4","tracks":[{"codec":"h264","info":{"type":"video","bit_depth":8,"chroma_subsampling":"4:2:0","dimensions":{"height":1,"width":2},"frames":0,"orientation":{"mirrored":false,"rotation":0}},"size":0},{"codec":"mp3","info":{"type":"audio","channels":2,"sample_rate":44100},"size":0}],"type":"mp4"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Video {
        placeholder: Some(Image::test("bar.png")),
        ..Video::test("foo.mp4")
      })
      .unwrap(),
      r#"{"duration":1000,"path":"foo.mp4","placeholder":{"alpha":false,"bit_depth":8,"color_type":"rgb","dimensions":{"height":1,"width":1},"orientation":{"mirrored":false,"rotation":0},"path":"bar.png","type":"png"},"tracks":[],"type":"mp4"}"#,
    );
  }

  #[test]
  fn sum_durations() {
    #[track_caller]
    fn case(durations: &[u64], expected: Duration) {
      let videos = durations
        .iter()
        .map(|duration| {
          let mut video = Item::<Video>::test("foo.mp4");
          video.content.duration = *duration;
          video
        })
        .collect::<Vec<Item<Video>>>();

      assert_eq!(Video::sum_durations(&videos), expected);
    }

    case(&[], Duration::ZERO);
    case(&[1500, 600], Duration::from_millis(2100));
  }
}
