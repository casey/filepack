use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Video {
  #[n(0)]
  pub(crate) filename: ComponentBuf,
  #[n(1)]
  pub(crate) tracks: Vec<Track>,
  #[n(2)]
  #[serde(rename = "type")]
  pub(crate) ty: VideoType,
}

impl Video {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  pub(crate) fn check_content(&self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    let tracks = self.info(root)?;

    self.check_info(&tracks).context(error::Video { path })
  }

  fn check_info(&self, tracks: &[Track]) -> Result<(), VideoError> {
    ensure! {
      tracks.len() == self.tracks.len(),
      video_error::TrackCountMismatch {
        actual: tracks.len(),
        expected: self.tracks.len(),
      },
    }

    for (index, (actual, expected)) in tracks.iter().zip(&self.tracks).enumerate() {
      ensure! {
        actual == expected,
        video_error::TrackMismatch {
          actual: *actual,
          expected: *expected,
          index,
        },
      }
    }

    Ok(())
  }

  pub(crate) fn dimensions(&self) -> Option<Dimensions> {
    self.tracks.iter().find_map(|track| match track.info {
      TrackInfo::Video { dimensions } => Some(dimensions),
      TrackInfo::Audio => None,
    })
  }

  fn format(&self) -> VideoFormat {
    VideoFormat {
      codecs: self.tracks.iter().map(|track| track.codec).collect(),
      ty: self.ty,
    }
  }

  pub(crate) fn formats(videos: &[Video]) -> Vec<VideoFormat> {
    let mut formats = Vec::new();

    for video in videos {
      let format = video.format();

      if !formats.contains(&format) {
        formats.push(format);
      }
    }

    formats
  }

  fn info(&self, root: &Utf8Path) -> Result<Vec<Track>> {
    let path = root.join(self.as_path());

    match self.ty {
      VideoType::Mp4 => {
        let file = filesystem::open(&path)?;

        let size = file
          .metadata()
          .context(error::FilesystemIo { path: &path })?
          .len();

        Self::info_mp4(file, size).context(error::Video { path })
      }
      VideoType::Webm => {
        let file = filesystem::open(&path)?;
        Self::info_webm(file).context(error::Video { path })
      }
    }
  }

  fn info_mp4<T: Read + Seek>(reader: T, size: u64) -> Result<Vec<Track>, VideoError> {
    use re_mp4::{Mp4, Mp4aBox, StsdBoxContent};

    fn mp4a_codec(mp4a: &Mp4aBox) -> Option<Codec> {
      match mp4a
        .esds
        .as_ref()?
        .es_desc
        .dec_config
        .object_type_indication
      {
        0x40 | 0x66 | 0x67 => Some(Codec::Aac),
        0x69 | 0x6b => Some(Codec::Mp3),
        _ => None,
      }
    }

    fn codec_name(contents: &StsdBoxContent) -> String {
      match contents {
        StsdBoxContent::Av01(_) => "AV1".into(),
        StsdBoxContent::Avc1(_) => "H.264".into(),
        StsdBoxContent::Hev1(_) | StsdBoxContent::Hvc1(_) => "H.265".into(),
        StsdBoxContent::Mp4a(mp4a) => match mp4a_codec(mp4a) {
          Some(codec) => codec.to_string(),
          None => "unknown".into(),
        },
        StsdBoxContent::Tx3g(_) => "TTXT".into(),
        StsdBoxContent::Unknown(fourcc) => fourcc.to_string(),
        StsdBoxContent::Vp08(_) => "VP8".into(),
        StsdBoxContent::Vp09(_) => "VP9".into(),
      }
    }

    let mp4 = Mp4::read(BufReader::new(reader), size).context(video_error::DecodeMp4)?;

    let mut video_codec = None;
    let mut dimensions = None;
    let mut audio_codec = None;

    for (index, trak) in mp4.moov.traks.iter().enumerate() {
      let contents = &trak.mdia.minf.stbl.stsd.contents;

      match &trak.mdia.hdlr.handler_type.value[..] {
        b"soun" => {
          ensure!(audio_codec.is_none(), video_error::AudioTrackMultiple);

          let codec = if let StsdBoxContent::Mp4a(mp4a) = contents {
            mp4a_codec(mp4a)
          } else {
            None
          };

          let Some(codec) = codec else {
            return Err(
              video_error::AudioCodecUnsupported {
                codec: codec_name(contents),
                track: index,
              }
              .build(),
            );
          };

          audio_codec = Some(codec);
        }
        b"vide" => {
          ensure!(video_codec.is_none(), video_error::VideoTrackMultiple);

          let StsdBoxContent::Avc1(avc1) = contents else {
            return Err(
              video_error::VideoCodecUnsupported {
                codec: codec_name(contents),
                track: index,
              }
              .build(),
            );
          };

          video_codec = Some(Codec::H264);

          dimensions = Some(Dimensions {
            height: avc1.height.into(),
            width: avc1.width.into(),
          });
        }
        ty => {
          return Err(
            video_error::TrackUnsupported {
              track: index,
              ty: match ty {
                b"auxv" => "auxiliary video",
                b"meta" => "metadata",
                b"pict" => "picture",
                _ => "unknown",
              },
            }
            .build(),
          );
        }
      }
    }

    let video_codec = video_codec.context(video_error::VideoTrackMissing)?;
    let dimensions = dimensions.unwrap();

    let mut tracks = vec![Track {
      codec: video_codec,
      info: TrackInfo::Video { dimensions },
    }];

    if let Some(audio_codec) = audio_codec {
      tracks.push(Track {
        codec: audio_codec,
        info: TrackInfo::Audio,
      });
    }

    Ok(tracks)
  }

  fn info_webm<T: Read + Seek>(reader: T) -> Result<Vec<Track>, VideoError> {
    use matroska_demuxer::{MatroskaFile, TrackType};

    let file = MatroskaFile::open(BufReader::new(reader)).context(video_error::DecodeWebm)?;

    let doc_type = file.ebml_header().doc_type().trim_end_matches('\0');

    ensure! {
      doc_type == "webm",
      video_error::DocType { doc_type },
    }

    let mut video_codec = None;
    let mut dimensions = None;
    let mut audio_codec = None;

    for (index, track) in file.tracks().iter().enumerate() {
      match track.track_type() {
        TrackType::Audio => {
          ensure!(audio_codec.is_none(), video_error::AudioTrackMultiple);

          audio_codec = Some(match track.codec_id() {
            "A_OPUS" => Codec::Opus,
            "A_VORBIS" => Codec::Vorbis,
            codec => {
              return Err(
                video_error::AudioCodecUnsupported {
                  codec,
                  track: index,
                }
                .build(),
              );
            }
          });
        }
        TrackType::Video => {
          ensure!(video_codec.is_none(), video_error::VideoTrackMultiple);

          video_codec = Some(match track.codec_id() {
            "V_VP8" => Codec::Vp8,
            "V_VP9" => Codec::Vp9,
            codec => {
              return Err(
                video_error::VideoCodecUnsupported {
                  codec,
                  track: index,
                }
                .build(),
              );
            }
          });

          let video = track
            .video()
            .context(video_error::VideoSettingsMissing { track: index })?;

          dimensions = Some(Dimensions {
            height: video.pixel_height().get(),
            width: video.pixel_width().get(),
          });
        }
        ty => {
          return Err(
            video_error::TrackUnsupported {
              track: index,
              ty: match ty {
                TrackType::Audio => "audio",
                TrackType::Buttons => "buttons",
                TrackType::Complex => "complex",
                TrackType::Control => "control",
                TrackType::Logo => "logo",
                TrackType::Metadata => "metadata",
                TrackType::Subtitle => "subtitle",
                TrackType::Unknown => "unknown",
                TrackType::Video => "video",
              },
            }
            .build(),
          );
        }
      }
    }

    let video_codec = video_codec.context(video_error::VideoTrackMissing)?;
    let dimensions = dimensions.unwrap();

    let mut tracks = vec![Track {
      codec: video_codec,
      info: TrackInfo::Video { dimensions },
    }];

    if let Some(audio_codec) = audio_codec {
      tracks.push(Track {
        codec: audio_codec,
        info: TrackInfo::Audio,
      });
    }

    Ok(tracks)
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    self.tracks = self.info(root)?;

    Ok(())
  }

  pub(crate) fn resolutions(videos: &[Video]) -> Option<Resolutions> {
    Resolutions::new(videos.iter().filter_map(Video::dimensions), true)
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
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
      filename,
      tracks: Vec::new(),
      ty,
    })
  }
}

impl Item for Video {
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
  fn check() {
    let mut video = "foo.mp4".parse::<Video>().unwrap();

    video.tracks = vec![
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions::default(),
        },
      },
      Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio,
      },
    ];

    video
      .check_info(&[
        Track {
          codec: Codec::H264,
          info: TrackInfo::Video {
            dimensions: Dimensions::default(),
          },
        },
        Track {
          codec: Codec::Aac,
          info: TrackInfo::Audio,
        },
      ])
      .unwrap();

    assert_eq!(
      video
        .check_info(&[Track {
          codec: Codec::H264,
          info: TrackInfo::Video {
            dimensions: Dimensions::default()
          },
        }])
        .unwrap_err()
        .to_string(),
      "video has 1 track but metadata has 2 tracks",
    );

    assert_eq!(
      video
        .check_info(&[
          Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              }
            },
          },
          Track {
            codec: Codec::Aac,
            info: TrackInfo::Audio,
          },
        ])
        .unwrap_err()
        .to_string(),
      "video track 0 `H.264 2×1` doesn't match metadata track `H.264 0×0`",
    );

    assert_eq!(
      video
        .check_info(&[
          Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              dimensions: Dimensions::default()
            },
          },
          Track {
            codec: Codec::Mp3,
            info: TrackInfo::Audio,
          },
        ])
        .unwrap_err()
        .to_string(),
      "video track 1 `MP3` doesn't match metadata track `AAC`",
    );
  }

  #[test]
  fn check_content() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.mp4"),
      Mp4Builder::new()
        .video_track(2, 1)
        .audio_track(0x40)
        .build(),
    )
    .unwrap();

    let mut video = "foo.mp4".parse::<Video>().unwrap();

    video.populate(&root).unwrap();

    video.check_content(&root).unwrap();

    video.tracks[0].info = TrackInfo::Video {
      dimensions: Dimensions {
        height: 4,
        width: 4,
      },
    };

    assert_matches_regex!(
      video.check_content(&root).unwrap_err().to_string(),
      r"^invalid video `.*foo\.mp4`$",
    );
  }

  #[test]
  fn dimensions() {
    let mut foo = "foo.mp4".parse::<Video>().unwrap();

    assert_eq!(foo.dimensions(), None);

    foo.tracks = vec![
      Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio,
      },
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
        },
      },
    ];

    assert_eq!(
      foo.dimensions(),
      Some(Dimensions {
        height: 1,
        width: 2,
      }),
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      "foo.mp4".parse::<Video>().unwrap(),
      "a30067666f6f2e6d703401800200",
    );

    assert_cbor(
      Video {
        filename: "foo.mp4".parse().unwrap(),
        tracks: vec![
          Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
            },
          },
          Track {
            codec: Codec::Mp3,
            info: TrackInfo::Audio,
          },
        ],
        ty: VideoType::Mp4,
      },
      "a30067666f6f2e6d70340182a20001018201a100a200010102a2000201000200",
    );
  }

  #[test]
  fn formats() {
    let mut foo = "foo.mp4".parse::<Video>().unwrap();

    foo.tracks = vec![
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions::default(),
        },
      },
      Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio,
      },
    ];

    let mut bar = foo.clone();
    bar.tracks[1].codec = Codec::Mp3;

    let mut baz = foo.clone();

    baz.tracks[0].info = TrackInfo::Video {
      dimensions: Dimensions {
        height: 1,
        width: 2,
      },
    };

    let mut bob = "bob.webm".parse::<Video>().unwrap();

    bob.tracks = vec![
      Track {
        codec: Codec::Vp9,
        info: TrackInfo::Video {
          dimensions: Dimensions::default(),
        },
      },
      Track {
        codec: Codec::Opus,
        info: TrackInfo::Audio,
      },
    ];

    assert_eq!(
      Video::formats(&[foo, bar, baz, bob])
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>(),
      [
        "MP4 · H.264 · AAC",
        "MP4 · H.264 · MP3",
        "WebM · VP9 · Opus"
      ],
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
        filename: "foo.mp4".parse().unwrap(),
        tracks: Vec::new(),
        ty: VideoType::Mp4,
      },
    );

    assert_eq!(
      "foo.webm".parse::<Video>().unwrap(),
      Video {
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
  fn mp4_info() {
    #[track_caller]
    fn case(builder: Mp4Builder) -> Result<Vec<Track>, VideoError> {
      let bytes = builder.build();
      let size = bytes.len().try_into().unwrap();
      Video::info_mp4(io::Cursor::new(bytes), size)
    }

    #[track_caller]
    fn error(builder: Mp4Builder, expected: &str) {
      assert_eq!(case(builder).unwrap_err().to_string(), expected);
    }

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1).audio_track(0x40)).unwrap(),
      vec![
        Track {
          codec: Codec::H264,
          info: TrackInfo::Video {
            dimensions: Dimensions {
              height: 1,
              width: 2,
            }
          },
        },
        Track {
          codec: Codec::Aac,
          info: TrackInfo::Audio,
        },
      ],
    );

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1)).unwrap(),
      vec![Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          }
        },
      }],
    );

    error(Mp4Builder::new().audio_track(0x40), "no video track");
    error(
      Mp4Builder::new()
        .video_track(2, 1)
        .video_track(2, 1)
        .audio_track(0x40),
      "multiple video tracks",
    );
    error(
      Mp4Builder::new()
        .video_track(2, 1)
        .audio_track(0x40)
        .audio_track(0x40),
      "multiple audio tracks",
    );
    error(
      Mp4Builder::new()
        .video_track(2, 1)
        .audio_track(0x40)
        .track(*b"meta", &[]),
      "track 2 has unsupported track type `metadata`",
    );
    error(
      Mp4Builder::new()
        .track(
          *b"vide",
          &[Mp4Builder::video_entry(*b"s263", *b"d263", 2, 1)],
        )
        .audio_track(0x40),
      "track 0 has unsupported video codec `s263`",
    );
    error(
      Mp4Builder::new().video_track(2, 1).audio_track(0x11),
      "track 1 has unsupported audio codec `unknown`",
    );

    assert_eq!(
      Video::info_mp4(io::Cursor::new(b"foo"), 3)
        .unwrap_err()
        .to_string(),
      "failed to decode MP4",
    );
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
          .video_track(2, 1)
          .audio_track(0x40)
          .build()
      )
      .unwrap(),
      Video {
        filename: "foo.mp4".parse().unwrap(),
        tracks: vec![
          Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              }
            },
          },
          Track {
            codec: Codec::Aac,
            info: TrackInfo::Audio,
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
            dimensions: Dimensions {
              height: 1,
              width: 2,
            }
          },
        },
        Track {
          codec: Codec::Mp3,
          info: TrackInfo::Audio,
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
        filename: "foo.mp4".parse().unwrap(),
        tracks: vec![
          Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              }
            },
          },
          Track {
            codec: Codec::Mp3,
            info: TrackInfo::Audio,
          },
        ],
        ty: VideoType::Mp4,
      })
      .unwrap(),
      r#"{"filename":"foo.mp4","tracks":[{"codec":"h264","info":{"type":"video","dimensions":{"height":1,"width":2}}},{"codec":"mp3","info":{"type":"audio"}}],"type":"mp4"}"#,
    );
  }

  #[test]
  fn webm_info() {
    #[track_caller]
    fn case(builder: WebmBuilder) -> Result<Vec<Track>, VideoError> {
      Video::info_webm(io::Cursor::new(builder.build()))
    }

    #[track_caller]
    fn error(builder: WebmBuilder, expected: &str) {
      assert_eq!(case(builder).unwrap_err().to_string(), expected);
    }

    assert_eq!(
      case(WebmBuilder::new().video_track(2, 1).audio_track("A_OPUS")).unwrap(),
      vec![
        Track {
          codec: Codec::Vp9,
          info: TrackInfo::Video {
            dimensions: Dimensions {
              height: 1,
              width: 2,
            }
          },
        },
        Track {
          codec: Codec::Opus,
          info: TrackInfo::Audio,
        },
      ],
    );

    assert_eq!(
      case(
        WebmBuilder::new()
          .track(1, "V_VP8", &WebmBuilder::video_settings(2, 1))
          .audio_track("A_VORBIS"),
      )
      .unwrap(),
      vec![
        Track {
          codec: Codec::Vp8,
          info: TrackInfo::Video {
            dimensions: Dimensions {
              height: 1,
              width: 2,
            }
          },
        },
        Track {
          codec: Codec::Vorbis,
          info: TrackInfo::Audio,
        },
      ],
    );

    assert_eq!(
      case(WebmBuilder::new().video_track(2, 1)).unwrap(),
      vec![Track {
        codec: Codec::Vp9,
        info: TrackInfo::Video {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          }
        },
      }],
    );

    error(WebmBuilder::new().audio_track("A_OPUS"), "no video track");
    error(
      WebmBuilder::new()
        .video_track(2, 1)
        .video_track(2, 1)
        .audio_track("A_OPUS"),
      "multiple video tracks",
    );
    error(
      WebmBuilder::new()
        .video_track(2, 1)
        .audio_track("A_OPUS")
        .audio_track("A_OPUS"),
      "multiple audio tracks",
    );
    error(
      WebmBuilder::new()
        .video_track(2, 1)
        .track(0x11, "S_TEXT/UTF8", &[]),
      "track 1 has unsupported track type `subtitle`",
    );
    error(
      WebmBuilder::new()
        .track(1, "V_MPEG4/ISO/AVC", &WebmBuilder::video_settings(2, 1))
        .audio_track("A_OPUS"),
      "track 0 has unsupported video codec `V_MPEG4/ISO/AVC`",
    );
    error(
      WebmBuilder::new().video_track(2, 1).audio_track("A_AAC"),
      "track 1 has unsupported audio codec `A_AAC`",
    );
    error(
      WebmBuilder::new().track(1, "V_VP9", &[]),
      "track 0 has missing video settings",
    );
    error(
      WebmBuilder::new().video_track(2, 1).doc_type("matroska"),
      "expected DocType `webm` but found `matroska`",
    );

    assert_eq!(
      Video::info_webm(io::Cursor::new(b"foo"))
        .unwrap_err()
        .to_string(),
      "failed to decode WebM",
    );
  }
}
