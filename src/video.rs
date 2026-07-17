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

  fn info(&self, root: &Utf8Path) -> Result<VideoInfo> {
    let path = root.join(self.as_path());

    match self.ty {
      VideoType::Mp4 => {
        let file = filesystem::open(&path)?;
        Self::decode_mp4(file).context(error::Video { path })
      }
    }
  }

  fn decode_mp4(file: fs::File) -> Result<VideoInfo, VideoError> {
    let reader = &mut BufReader::new(file);
    let context = mp4parse::read_mp4(reader).context(video_error::Decode)?;

    let mut audio = None;
    let mut video = None;

    for track in &context.tracks {
      match &track.track_type {
        mp4parse::TrackType::Audio => {
          ensure!(audio.is_none(), video_error::AudioTrackMultiple);
          audio = Some(track);
        }
        mp4parse::TrackType::Video => {
          ensure!(video.is_none(), video_error::VideoTrackMultiple);
          video = Some(track);
        }
        ty => {
          return video_error::TrackUnsupported {
            ty: format!("{ty:?}"),
          }
          .fail();
        }
      }
    }

    let video = video.context(video_error::VideoTrackMissing)?;
    let audio = audio.context(video_error::AudioTrackMissing)?;

    let mp4parse::SampleEntry::Video(video) = Self::description(video)? else {
      return video_error::VideoCodecUnsupported { codec: "unknown" }.fail();
    };

    let video_codec = match video.codec_type {
      mp4parse::CodecType::H263 => VideoCodec::H263,
      codec => {
        return video_error::VideoCodecUnsupported {
          codec: format!("{codec:?}"),
        }
        .fail();
      }
    };

    let dimensions = Dimensions {
      height: video.height.into(),
      width: video.width.into(),
    };

    let mp4parse::SampleEntry::Audio(audio) = Self::description(audio)? else {
      return video_error::AudioCodecUnsupported { codec: "unknown" }.fail();
    };

    let audio_codec = match audio.codec_type {
      mp4parse::CodecType::AAC => AudioCodec::Aac,
      mp4parse::CodecType::MP3 => AudioCodec::Mp3,
      codec => {
        return video_error::AudioCodecUnsupported {
          codec: format!("{codec:?}"),
        }
        .fail();
      }
    };

    Ok(VideoInfo {
      audio_codec,
      dimensions,
      video_codec,
    })
  }

  fn description(track: &mp4parse::Track) -> Result<&mp4parse::SampleEntry, VideoError> {
    let descriptions = track
      .stsd
      .as_ref()
      .map(|stsd| &*stsd.descriptions)
      .unwrap_or_default();

    ensure! {
      descriptions.len() == 1,
      video_error::SampleDescriptions {
        count: descriptions.len(),
      },
    }

    Ok(&descriptions[0])
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
  fn decode_mp4() {
    #[track_caller]
    fn case(builder: VideoBuilder) -> Result<VideoInfo, VideoError> {
      Video::decode_mp4(&mut io::Cursor::new(builder.build()))
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
      "unsupported track type `Metadata`",
    );
    error(
      VideoBuilder::new()
        .track(
          *b"vide",
          &[VideoBuilder::video_entry(*b"avc1", *b"avcC", 2, 1)],
        )
        .audio_track(0x40),
      "unsupported video codec `H264`",
    );
    error(
      VideoBuilder::new().video_track(2, 1).audio_track(0x11),
      "unsupported audio codec `Unknown`",
    );
    error(
      VideoBuilder::new().video_track(2, 1).track(
        *b"soun",
        &[
          VideoBuilder::audio_entry(0x40),
          VideoBuilder::audio_entry(0x40),
        ],
      ),
      "track has 2 sample descriptions",
    );
    error(
      VideoBuilder::new().video_track(2, 1).track(*b"soun", &[]),
      "track has 0 sample descriptions",
    );

    assert_eq!(
      Video::decode_mp4(&mut io::Cursor::new(b"foo"))
        .unwrap_err()
        .to_string(),
      "failed to decode MP4",
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
