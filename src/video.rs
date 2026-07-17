use super::*;

#[derive(Debug, PartialEq)]
struct Mp4Info {
  audio_codec: AudioCodec,
  dimensions: Dimensions,
  video_codec: VideoCodec,
}

impl Mp4Info {
  fn check(&self, video: &Video) -> Result<(), VideoError> {
    ensure! {
      self.dimensions == video.dimensions,
      video_error::DimensionsMismatch {
        actual: self.dimensions,
        expected: video.dimensions,
      },
    }

    ensure! {
      self.video_codec == video.video_codec,
      video_error::VideoCodecMismatch {
        actual: self.video_codec,
        expected: video.video_codec,
      },
    }

    ensure! {
      self.audio_codec == video.audio_codec,
      video_error::AudioCodecMismatch {
        actual: self.audio_codec,
        expected: video.audio_codec,
      },
    }

    Ok(())
  }

  fn decode(bytes: Vec<u8>) -> Result<Self, VideoError> {
    let context = mp4parse::read_mp4(&mut io::Cursor::new(bytes)).context(video_error::Decode)?;

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

    Ok(Self {
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
}

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

  pub(crate) fn check_content(&self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    let info = self.decode(root)?;

    info.check(self).context(error::Video { path })
  }

  fn decode(&self, root: &Utf8Path) -> Result<Mp4Info> {
    let path = root.join(self.as_path());

    match self.ty {
      VideoType::Mp4 => Self::decode_mp4(&path),
    }
  }

  fn decode_mp4(path: &Utf8Path) -> Result<Mp4Info> {
    let bytes = filesystem::read(path)?;

    Mp4Info::decode(bytes).context(error::Video { path })
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
    let info = self.decode(root)?;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VideoFormat {
  audio_codec: AudioCodec,
  ty: VideoType,
  video_codec: VideoCodec,
}

impl Display for VideoFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{} / {} / {}",
      self.ty, self.video_codec, self.audio_codec
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn atom(fourcc: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut atom = Vec::new();
    atom.extend_from_slice(&u32::try_from(payload.len() + 8).unwrap().to_be_bytes());
    atom.extend_from_slice(&fourcc);
    atom.extend_from_slice(payload);
    atom
  }

  fn audio_entry(object_type: u8) -> Vec<u8> {
    let mut descriptor = vec![0x04, 13, object_type];
    descriptor.extend_from_slice(&[0; 12]);

    let mut es = vec![0x03, u8::try_from(descriptor.len() + 3).unwrap(), 0, 1, 0];
    es.extend_from_slice(&descriptor);

    let mut esds = vec![0, 0, 0, 0];
    esds.extend_from_slice(&es);

    let mut payload = Vec::new();
    payload.extend_from_slice(&[0; 6]);
    payload.extend_from_slice(&[0, 1]);
    payload.extend_from_slice(&[0; 8]);
    payload.extend_from_slice(&2u16.to_be_bytes());
    payload.extend_from_slice(&16u16.to_be_bytes());
    payload.extend_from_slice(&[0; 4]);
    payload.extend_from_slice(&(44100u32 << 16).to_be_bytes());
    payload.extend_from_slice(&atom(*b"esds", &esds));

    atom(*b"mp4a", &payload)
  }

  #[test]
  fn check() {
    let video = "foo.mp4".parse::<Video>().unwrap();

    Mp4Info {
      audio_codec: AudioCodec::Aac,
      dimensions: Dimensions::default(),
      video_codec: VideoCodec::H263,
    }
    .check(&video)
    .unwrap();

    assert_eq!(
      Mp4Info {
        audio_codec: AudioCodec::Aac,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        video_codec: VideoCodec::H263,
      }
      .check(&video)
      .unwrap_err()
      .to_string(),
      "video is 2×1 but metadata dimensions are 0×0",
    );

    assert_eq!(
      Mp4Info {
        audio_codec: AudioCodec::Mp3,
        dimensions: Dimensions::default(),
        video_codec: VideoCodec::H263,
      }
      .check(&video)
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
      mp4(&[
        trak(*b"vide", &[video_entry(*b"s263", *b"d263", 2, 1)]),
        trak(*b"soun", &[audio_entry(0x40)]),
      ]),
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
  fn decode() {
    #[track_caller]
    fn case(traks: &[Vec<u8>]) -> Result<Mp4Info, VideoError> {
      Mp4Info::decode(mp4(traks))
    }

    #[track_caller]
    fn error(traks: &[Vec<u8>], expected: &str) {
      assert_eq!(case(traks).unwrap_err().to_string(), expected);
    }

    let video = trak(*b"vide", &[video_entry(*b"s263", *b"d263", 2, 1)]);
    let audio = trak(*b"soun", &[audio_entry(0x40)]);

    assert_eq!(
      case(&[video.clone(), audio.clone()]).unwrap(),
      Mp4Info {
        audio_codec: AudioCodec::Aac,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        video_codec: VideoCodec::H263,
      },
    );

    error(std::slice::from_ref(&audio), "no video track");
    error(std::slice::from_ref(&video), "no audio track");
    error(
      &[video.clone(), video.clone(), audio.clone()],
      "multiple video tracks",
    );
    error(
      &[video.clone(), audio.clone(), audio.clone()],
      "multiple audio tracks",
    );
    error(
      &[video.clone(), audio.clone(), trak(*b"meta", &[])],
      "unsupported track type `Metadata`",
    );
    error(
      &[
        trak(*b"vide", &[video_entry(*b"avc1", *b"avcC", 2, 1)]),
        audio.clone(),
      ],
      "unsupported video codec `H264`",
    );
    error(
      &[video.clone(), trak(*b"soun", &[audio_entry(0x11)])],
      "unsupported audio codec `Unknown`",
    );
    error(
      &[
        video.clone(),
        trak(*b"soun", &[audio_entry(0x40), audio_entry(0x40)]),
      ],
      "track has 2 sample descriptions",
    );
    error(
      &[video.clone(), trak(*b"soun", &[])],
      "track has 0 sample descriptions",
    );

    assert_eq!(
      Mp4Info::decode(b"foo".to_vec()).unwrap_err().to_string(),
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
      ["MP4 / H263 / AAC", "MP4 / H263 / MP3"],
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

  fn mp4(traks: &[Vec<u8>]) -> Vec<u8> {
    let mut ftyp = Vec::new();
    ftyp.extend_from_slice(b"isom");
    ftyp.extend_from_slice(&[0; 4]);
    ftyp.extend_from_slice(b"isom");

    [atom(*b"ftyp", &ftyp), atom(*b"moov", &traks.concat())].concat()
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
      case(&mp4(&[
        trak(*b"vide", &[video_entry(*b"s263", *b"d263", 2, 1)]),
        trak(*b"soun", &[audio_entry(0x40)]),
      ]))
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
      case(&mp4(&[
        trak(*b"vide", &[video_entry(*b"s263", *b"d263", 2, 1)]),
        trak(*b"soun", &[audio_entry(0x6b)]),
      ]))
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

  fn trak(handler: [u8; 4], descriptions: &[Vec<u8>]) -> Vec<u8> {
    let mut hdlr = vec![0; 8];
    hdlr.extend_from_slice(&handler);
    hdlr.extend_from_slice(&[0; 12]);
    hdlr.push(0);

    let mut stsd = vec![0, 0, 0, 0];
    stsd.extend_from_slice(&u32::try_from(descriptions.len()).unwrap().to_be_bytes());
    stsd.extend_from_slice(&descriptions.concat());

    let stbl = atom(*b"stbl", &atom(*b"stsd", &stsd));
    let minf = atom(*b"minf", &stbl);
    let mdia = [atom(*b"hdlr", &hdlr), minf].concat();

    atom(*b"trak", &atom(*b"mdia", &mdia))
  }

  fn video_entry(entry: [u8; 4], config: [u8; 4], width: u16, height: u16) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0; 6]);
    payload.extend_from_slice(&[0, 1]);
    payload.extend_from_slice(&[0; 16]);
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    payload.extend_from_slice(&[0; 50]);
    payload.extend_from_slice(&atom(config, &[]));

    atom(entry, &payload)
  }
}
