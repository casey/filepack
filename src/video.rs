use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Video {
  #[n(0)]
  pub(crate) audio_codec: AudioCodec,
  #[n(1)]
  pub(crate) dimensions: Dimensions,
  #[n(2)]
  pub(crate) filename: ComponentBuf,
  #[n(3)]
  #[serde(rename = "type")]
  pub(crate) ty: VideoType,
  #[n(4)]
  pub(crate) video_codec: VideoCodec,
}

impl Video {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  fn check(&self, info: &VideoInfo) -> Result<(), VideoError> {
    ensure! {
      self.dimensions == info.dimensions,
      video_error::DimensionsMismatch {
        actual: info.dimensions,
        expected: self.dimensions,
      },
    }

    ensure! {
      self.video_codec == info.video_codec,
      video_error::VideoCodecMismatch {
        actual: info.video_codec,
        expected: self.video_codec,
      },
    }

    ensure! {
      self.audio_codec == info.audio_codec,
      video_error::AudioCodecMismatch {
        actual: info.audio_codec,
        expected: self.audio_codec,
      },
    }

    Ok(())
  }

  pub(crate) fn check_content(&self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    let info = self.info(root)?;

    self.check(&info).context(error::Video { path })
  }

  pub(crate) fn format(&self) -> VideoFormat {
    VideoFormat {
      audio_codec: self.audio_codec,
      ty: self.ty,
      video_codec: self.video_codec,
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

  fn info(&self, root: &Utf8Path) -> Result<VideoInfo> {
    let path = root.join(self.as_path());

    match self.ty {
      VideoType::Mp4 => {
        let file = filesystem::open(&path)?;
        Self::info_mp4(file).context(error::Video { path })
      }
    }
  }

  fn info_mp4<T: Read>(reader: T) -> Result<VideoInfo, VideoError> {
    use mp4parse::{CodecType, SampleEntry, TrackType};

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
        TrackType::Audio => {
          ensure!(audio_codec.is_none(), video_error::AudioTrackMultiple);

          let SampleEntry::Audio(audio) = Self::track_description(track)? else {
            return video_error::AudioCodecUnsupported {
              codec: "unknown",
              track: track.id,
            }
            .fail();
          };

          audio_codec = Some(match audio.codec_type {
            CodecType::AAC => AudioCodec::Aac,
            CodecType::MP3 => AudioCodec::Mp3,
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
        TrackType::Video => {
          ensure!(video_codec.is_none(), video_error::VideoTrackMultiple);
          let SampleEntry::Video(video) = Self::track_description(track)? else {
            return video_error::VideoCodecUnsupported {
              codec: "unknown",
              track: track.id,
            }
            .fail();
          };

          video_codec = Some(match video.codec_type {
            CodecType::H263 => VideoCodec::H263,
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
                TrackType::Audio => "audio",
                TrackType::AuxiliaryVideo => "auixiliary video",
                TrackType::Metadata => "metadata",
                TrackType::Picture => "picture",
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
    let audio_codec = audio_codec.context(video_error::AudioTrackMissing)?;

    Ok(VideoInfo {
      audio_codec,
      dimensions,
      video_codec,
    })
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let info = self.info(root)?;

    self.audio_codec = info.audio_codec;
    self.dimensions = info.dimensions;
    self.video_codec = info.video_codec;

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
      audio_codec: AudioCodec::Aac,
      dimensions: Dimensions::default(),
      filename,
      ty,
      video_codec: VideoCodec::H263,
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
      .check(&VideoInfo {
        audio_codec: AudioCodec::Aac,
        dimensions: Dimensions::default(),
        video_codec: VideoCodec::H263,
      })
      .unwrap();

    assert_eq!(
      video
        .check(&VideoInfo {
          audio_codec: AudioCodec::Aac,
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          video_codec: VideoCodec::H263,
        })
        .unwrap_err()
        .to_string(),
      "video is 2×1 but metadata dimensions are 0×0",
    );

    assert_eq!(
      video
        .check(&VideoInfo {
          audio_codec: AudioCodec::Mp3,
          dimensions: Dimensions::default(),
          video_codec: VideoCodec::H263,
        })
        .unwrap_err()
        .to_string(),
      "audio codec MP3 doesn't match metadata audio codec AAC",
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

    video.dimensions = Dimensions {
      height: 4,
      width: 4,
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
      "a5000001a2000001000267666f6f2e6d703403000400",
    );

    assert_cbor(
      Video {
        audio_codec: AudioCodec::Mp3,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.mp4".parse().unwrap(),
        ty: VideoType::Mp4,
        video_codec: VideoCodec::H263,
      },
      "a5000101a2000101020267666f6f2e6d703403000400",
    );
  }

  #[test]
  fn formats() {
    let foo = "foo.mp4".parse::<Video>().unwrap();
    let mut bar = "bar.mp4".parse::<Video>().unwrap();
    bar.audio_codec = AudioCodec::Mp3;
    let baz = "baz.mp4".parse::<Video>().unwrap();

    assert_eq!(
      Video::formats(&[foo, bar, baz])
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>(),
      ["MP4 H263 AAC", "MP4 H263 MP3"],
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
        audio_codec: AudioCodec::Aac,
        dimensions: Dimensions::default(),
        filename: "foo.mp4".parse().unwrap(),
        ty: VideoType::Mp4,
        video_codec: VideoCodec::H263,
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
    fn case(builder: VideoBuilder) -> Result<VideoInfo, VideoError> {
      Video::info_mp4(io::Cursor::new(builder.build()))
    }

    #[track_caller]
    fn error(builder: VideoBuilder, expected: &str) {
      assert_eq!(case(builder).unwrap_err().to_string(), expected);
    }

    assert_eq!(
      case(VideoBuilder::new().video_track(2, 1).audio_track(0x40)).unwrap(),
      VideoInfo {
        audio_codec: AudioCodec::Aac,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        video_codec: VideoCodec::H263,
      },
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
          &[VideoBuilder::video_entry(*b"avc1", *b"avcC", 2, 1)],
        )
        .audio_track(0x40),
      "track 0 has unsupported video codec `H.264`",
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
        audio_codec: AudioCodec::Aac,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.mp4".parse().unwrap(),
        ty: VideoType::Mp4,
        video_codec: VideoCodec::H263,
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
      .audio_codec,
      AudioCodec::Mp3,
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
        audio_codec: AudioCodec::Mp3,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        filename: "foo.mp4".parse().unwrap(),
        ty: VideoType::Mp4,
        video_codec: VideoCodec::H263,
      })
      .unwrap(),
      r#"{"audio_codec":"mp3","dimensions":{"height":1,"width":2},"filename":"foo.mp4","type":"mp4","video_codec":"h263"}"#,
    );
  }
}
