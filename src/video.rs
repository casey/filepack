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

  fn format(&self) -> Option<VideoFormat> {
    let video_codec = self.tracks.iter().find_map(|track| match track.ty {
      TrackType::Video { .. } => Some(track.codec),
      TrackType::Audio => None,
    })?;

    let audio_codec = self.tracks.iter().find_map(|track| match track.ty {
      TrackType::Audio => Some(track.codec),
      TrackType::Video { .. } => None,
    })?;

    Some(VideoFormat {
      audio_codec,
      ty: self.ty,
      video_codec,
    })
  }

  pub(crate) fn formats(videos: &[Video]) -> Vec<VideoFormat> {
    let mut formats = Vec::new();

    for video in videos {
      if let Some(format) = video.format()
        && !formats.contains(&format)
      {
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
        Self::info_mp4(file).context(error::Video { path })
      }
    }
  }

  fn info_mp4<T: Read>(reader: T) -> Result<Vec<Track>, VideoError> {
    use mp4parse::{CodecType, SampleEntry};

    fn codec_name(ty: CodecType) -> &'static str {
      match ty {
        CodecType::AAC => "AAC",
        CodecType::ALAC => "ALAC",
        CodecType::AV1 => "AV1",
        CodecType::EncryptedAudio => "encrypted audio",
        CodecType::EncryptedVideo => "encrypted video",
        CodecType::FLAC => "FLAC",
        CodecType::H263 => "H.263",
        CodecType::H264 => "H.264",
        CodecType::LPCM => "LPCM",
        CodecType::MP3 => "MP3",
        CodecType::MP4V => "MP4V",
        CodecType::Opus => "Opus",
        CodecType::Unknown => "unknown",
        CodecType::VP8 => "VP8",
        CodecType::VP9 => "VP9",
      }
    }

    let reader = &mut BufReader::new(reader);

    let context = mp4parse::read_mp4(reader).context(video_error::DecodeMp4)?;

    let mut video_codec = None;
    let mut dimensions = None;
    let mut audio_codec = None;

    for track in &context.tracks {
      match &track.track_type {
        mp4parse::TrackType::Audio => {
          ensure!(audio_codec.is_none(), video_error::AudioTrackMultiple);

          let SampleEntry::Audio(audio) = Self::track_description(track)? else {
            return video_error::AudioCodecUnsupported {
              codec: "unknown",
              track: track.id,
            }
            .fail();
          };

          audio_codec = Some(match audio.codec_type {
            CodecType::AAC => Codec::Aac,
            CodecType::MP3 => Codec::Mp3,
            codec => {
              return Err(
                video_error::AudioCodecUnsupported {
                  codec: codec_name(codec),
                  track: track.id,
                }
                .build(),
              );
            }
          });
        }
        mp4parse::TrackType::Video => {
          ensure!(video_codec.is_none(), video_error::VideoTrackMultiple);
          let SampleEntry::Video(video) = Self::track_description(track)? else {
            return video_error::VideoCodecUnsupported {
              codec: "unknown",
              track: track.id,
            }
            .fail();
          };

          video_codec = Some(match video.codec_type {
            CodecType::H264 => Codec::H264,
            codec => {
              return Err(
                video_error::VideoCodecUnsupported {
                  codec: codec_name(codec),
                  track: track.id,
                }
                .build(),
              );
            }
          });

          dimensions = Some(Dimensions {
            height: video.height.into(),
            width: video.width.into(),
          });
        }
        ty => {
          return Err(
            video_error::TrackUnsupported {
              track: track.id,
              ty: match ty {
                mp4parse::TrackType::Audio => "audio",
                mp4parse::TrackType::AuxiliaryVideo => "auixiliary video",
                mp4parse::TrackType::Metadata => "metadata",
                mp4parse::TrackType::Picture => "picture",
                mp4parse::TrackType::Unknown => "unknown",
                mp4parse::TrackType::Video => "video",
              },
            }
            .build(),
          );
        }
      }
    }

    let video_codec = video_codec.context(video_error::VideoTrackMissing)?;
    let dimensions = dimensions.unwrap();
    let audio_codec = audio_codec.context(video_error::AudioTrackMissing)?;

    Ok(vec![
      Track {
        codec: video_codec,
        ty: TrackType::Video { dimensions },
      },
      Track {
        codec: audio_codec,
        ty: TrackType::Audio,
      },
    ])
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    self.tracks = self.info(root)?;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }

  fn track_description(track: &mp4parse::Track) -> Result<&mp4parse::SampleEntry, VideoError> {
    let descriptions = track
      .stsd
      .as_ref()
      .map(|stsd| &*stsd.descriptions)
      .context(video_error::SampleDescriptionMissing { track: track.id })?;

    if descriptions.len() > 1 {
      return Err(video_error::SampleDescriptionMultiple { track: track.id }.build());
    }

    descriptions
      .first()
      .context(video_error::SampleDescriptionMissing { track: track.id })
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
      tracks: vec![
        Track {
          codec: Codec::H264,
          ty: TrackType::Video {
            dimensions: Dimensions::default(),
          },
        },
        Track {
          codec: Codec::Aac,
          ty: TrackType::Audio,
        },
      ],
      ty,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check() {
    let video = "foo.mp4".parse::<Video>().unwrap();

    video
      .check_info(&[
        Track {
          codec: Codec::H264,
          ty: TrackType::Video {
            dimensions: Dimensions::default(),
          },
        },
        Track {
          codec: Codec::Aac,
          ty: TrackType::Audio,
        },
      ])
      .unwrap();

    assert_eq!(
      video
        .check_info(&[Track {
          codec: Codec::H264,
          ty: TrackType::Video {
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
            ty: TrackType::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              }
            },
          },
          Track {
            codec: Codec::Aac,
            ty: TrackType::Audio,
          },
        ])
        .unwrap_err()
        .to_string(),
      "video track 0 `H264 2×1` doesn't match metadata track `H264 0×0`",
    );

    assert_eq!(
      video
        .check_info(&[
          Track {
            codec: Codec::H264,
            ty: TrackType::Video {
              dimensions: Dimensions::default()
            },
          },
          Track {
            codec: Codec::Mp3,
            ty: TrackType::Audio,
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
      VideoBuilder::new()
        .video_track(2, 1)
        .audio_track(0x40)
        .build(),
    )
    .unwrap();

    let mut video = "foo.mp4".parse::<Video>().unwrap();

    video.populate(&root).unwrap();

    video.check_content(&root).unwrap();

    video.tracks[0].ty = TrackType::Video {
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
  fn encoding() {
    assert_cbor(
      "foo.mp4".parse::<Video>().unwrap(),
      "a30067666f6f2e6d70340182a20001018201a100a200000100a2000001000200",
    );

    assert_cbor(
      Video {
        filename: "foo.mp4".parse().unwrap(),
        tracks: vec![
          Track {
            codec: Codec::H264,
            ty: TrackType::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
            },
          },
          Track {
            codec: Codec::Mp3,
            ty: TrackType::Audio,
          },
        ],
        ty: VideoType::Mp4,
      },
      "a30067666f6f2e6d70340182a20001018201a100a200010102a2000201000200",
    );
  }

  #[test]
  fn formats() {
    let foo = "foo.mp4".parse::<Video>().unwrap();
    let mut bar = "bar.mp4".parse::<Video>().unwrap();
    bar.tracks[1].codec = Codec::Mp3;
    let baz = "baz.mp4".parse::<Video>().unwrap();

    assert_eq!(
      Video::formats(&[foo, bar, baz])
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>(),
      ["MP4 H264 AAC", "MP4 H264 MP3"],
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
        tracks: vec![
          Track {
            codec: Codec::H264,
            ty: TrackType::Video {
              dimensions: Dimensions::default()
            },
          },
          Track {
            codec: Codec::Aac,
            ty: TrackType::Audio,
          },
        ],
        ty: VideoType::Mp4,
      },
    );

    case(
      "foo.avi",
      ComponentError::Extension {
        extensions: &["mp4"],
      },
    );
    case(
      "foo",
      ComponentError::Extension {
        extensions: &["mp4"],
      },
    );
    case("", ComponentError::Empty);
    case("foo/bar.mp4", ComponentError::Separator { character: '/' });
  }

  #[test]
  fn mp4_info() {
    #[track_caller]
    fn case(builder: VideoBuilder) -> Result<Vec<Track>, VideoError> {
      Video::info_mp4(io::Cursor::new(builder.build()))
    }

    #[track_caller]
    fn error(builder: VideoBuilder, expected: &str) {
      assert_eq!(case(builder).unwrap_err().to_string(), expected);
    }

    assert_eq!(
      case(VideoBuilder::new().video_track(2, 1).audio_track(0x40)).unwrap(),
      vec![
        Track {
          codec: Codec::H264,
          ty: TrackType::Video {
            dimensions: Dimensions {
              height: 1,
              width: 2,
            }
          },
        },
        Track {
          codec: Codec::Aac,
          ty: TrackType::Audio,
        },
      ],
    );

    error(VideoBuilder::new().audio_track(0x40), "no video track");
    error(VideoBuilder::new().video_track(2, 1), "no audio track");
    error(
      VideoBuilder::new()
        .video_track(2, 1)
        .video_track(2, 1)
        .audio_track(0x40),
      "multiple video tracks",
    );
    error(
      VideoBuilder::new()
        .video_track(2, 1)
        .audio_track(0x40)
        .audio_track(0x40),
      "multiple audio tracks",
    );
    error(
      VideoBuilder::new()
        .video_track(2, 1)
        .audio_track(0x40)
        .track(*b"meta", &[]),
      "track 2 has unsupported track type `metadata`",
    );
    error(
      VideoBuilder::new()
        .track(
          *b"vide",
          &[VideoBuilder::video_entry(*b"s263", *b"d263", 2, 1)],
        )
        .audio_track(0x40),
      "track 0 has unsupported video codec `H.263`",
    );
    error(
      VideoBuilder::new().video_track(2, 1).audio_track(0x11),
      "track 1 has unsupported audio codec `unknown`",
    );
    error(
      VideoBuilder::new().video_track(2, 1).track(
        *b"soun",
        &[
          VideoBuilder::audio_entry(0x40),
          VideoBuilder::audio_entry(0x40),
        ],
      ),
      "track 1 has multiple sample descriptions",
    );
    error(
      VideoBuilder::new().video_track(2, 1).track(*b"soun", &[]),
      "track 1 has missing sample description",
    );

    assert_eq!(
      Video::info_mp4(io::Cursor::new(b"foo"))
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
        &VideoBuilder::new()
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
            ty: TrackType::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              }
            },
          },
          Track {
            codec: Codec::Aac,
            ty: TrackType::Audio,
          },
        ],
        ty: VideoType::Mp4,
      },
    );

    assert_eq!(
      case(
        &VideoBuilder::new()
          .video_track(2, 1)
          .audio_track(0x6b)
          .build()
      )
      .unwrap()
      .tracks,
      vec![
        Track {
          codec: Codec::H264,
          ty: TrackType::Video {
            dimensions: Dimensions {
              height: 1,
              width: 2,
            }
          },
        },
        Track {
          codec: Codec::Mp3,
          ty: TrackType::Audio,
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
            ty: TrackType::Video {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              }
            },
          },
          Track {
            codec: Codec::Mp3,
            ty: TrackType::Audio,
          },
        ],
        ty: VideoType::Mp4,
      })
      .unwrap(),
      r#"{"filename":"foo.mp4","tracks":[{"codec":"h264","type":{"type":"video","dimensions":{"height":1,"width":2}}},{"codec":"mp3","type":{"type":"audio"}}],"type":"mp4"}"#,
    );
  }
}
