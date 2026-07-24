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

  pub(crate) fn dimensions(&self) -> Option<Dimensions> {
    self.tracks.iter().find_map(|track| match track.info {
      TrackInfo::Video { dimensions, .. } => Some(dimensions),
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

  fn h264_bit_depth(sps: &[u8]) -> Option<u64> {
    let mut rbsp = Vec::new();

    // skip NAL unit header and remove emulation prevention bytes
    for &byte in sps.get(1..)? {
      if byte == 3 && rbsp.ends_with(&[0, 0]) {
        continue;
      }

      rbsp.push(byte);
    }

    let mut reader = BitReader::new(&rbsp);

    // profile_idc
    let profile_idc = reader.bits(8)?;

    // constraint flags and level_idc
    reader.bits(16)?;

    // seq_parameter_set_id
    reader.ue()?;

    if !Self::h264_high_profile(profile_idc) {
      return Some(8);
    }

    // chroma_format_idc
    if reader.ue()? == 3 {
      // separate_colour_plane_flag
      reader.bit()?;
    }

    // bit_depth_luma_minus8
    Some(8 + reader.ue()?)
  }

  fn h264_high_profile(profile_idc: u64) -> bool {
    matches!(
      profile_idc,
      44 | 83 | 86 | 100 | 110 | 118 | 122 | 128 | 134 | 135 | 138 | 139 | 244
    )
  }

  fn info(&self, root: &Utf8Path) -> Result<VideoInfo> {
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

  fn info_mp4<T: Read + Seek>(reader: T, size: u64) -> Result<VideoInfo, VideoError> {
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

    let mvhd = &mp4.moov.mvhd;

    ensure!(mvhd.timescale != 0, video_error::TimescaleZero);

    let duration = u64::try_from(u128::from(mvhd.duration) * 1000 / u128::from(mvhd.timescale))
      .ok()
      .context(video_error::DurationOverflow)?;

    let mut video_track = None;
    let mut audio_track = None;

    for (index, trak) in mp4.moov.traks.iter().enumerate() {
      let contents = &trak.mdia.minf.stbl.stsd.contents;

      let stsz = &trak.mdia.minf.stbl.stsz;

      let size = if stsz.sample_size == 0 {
        stsz.sample_sizes.iter().copied().map(u64::from).sum()
      } else {
        u64::from(stsz.sample_size) * u64::from(stsz.sample_count)
      };

      match &trak.mdia.hdlr.handler_type.value[..] {
        b"soun" => {
          ensure!(audio_track.is_none(), video_error::AudioTrackMultiple);

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

          audio_track = Some(Track {
            codec,
            info: TrackInfo::Audio,
            size,
          });
        }
        b"vide" => {
          ensure!(video_track.is_none(), video_error::VideoTrackMultiple);

          let StsdBoxContent::Avc1(avc1) = contents else {
            return Err(
              video_error::VideoCodecUnsupported {
                codec: codec_name(contents),
                track: index,
              }
              .build(),
            );
          };

          let bit_depth = if let Some(sps) = avc1.avcc.sequence_parameter_sets.first() {
            Some(Self::h264_bit_depth(&sps.bytes).context(video_error::SpsInvalid)?)
          } else {
            ensure!(
              !Self::h264_high_profile(avc1.avcc.avc_profile_indication.into()),
              video_error::SpsMissing,
            );

            Some(8)
          };

          video_track = Some(Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              bit_depth,
              dimensions: Dimensions {
                height: avc1.height.into(),
                width: avc1.width.into(),
              },
              frames: stsz.sample_count.into(),
            },
            size,
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

    let mut tracks = vec![video_track.context(video_error::VideoTrackMissing)?];

    if let Some(track) = audio_track {
      tracks.push(track);
    }

    Ok(VideoInfo { duration, tracks })
  }

  fn info_webm<T: Read + Seek>(reader: T) -> Result<VideoInfo, VideoError> {
    use matroska_demuxer::{Frame, MatroskaFile, TrackType};

    let mut file = MatroskaFile::open(BufReader::new(reader)).context(video_error::DecodeWebm)?;

    let doc_type = file.ebml_header().doc_type().trim_end_matches('\0');

    ensure! {
      doc_type == "webm",
      video_error::DocType { doc_type },
    }

    let info = file.info();

    let ticks = info.duration().context(video_error::DurationMissing)?;

    let timestamp_scale = info.timestamp_scale().get();

    let timestamp_scale = u32::try_from(timestamp_scale)
      .ok()
      .context(video_error::TimestampScale { timestamp_scale })?;

    let duration = Duration::try_from_secs_f64(ticks * f64::from(timestamp_scale) / 1e9)
      .ok()
      .context(video_error::DurationInvalid)?;

    let duration = u64::try_from(duration.as_millis())
      .ok()
      .context(video_error::DurationOverflow)?;

    let mut frame = Frame::default();

    let mut frames = HashMap::<u64, (u64, u64, Option<Vec<u8>>)>::new();

    while file
      .next_frame(&mut frame)
      .context(video_error::DecodeWebm)?
    {
      let (count, size, first) = frames.entry(frame.track).or_default();
      *count += 1;
      *size += frame.data.len().into_u64();
      first.get_or_insert_with(|| frame.data.clone());
    }

    let mut video_track = None;
    let mut audio_track = None;

    for (index, track) in file.tracks().iter().enumerate() {
      match track.track_type() {
        TrackType::Audio => {
          ensure!(audio_track.is_none(), video_error::AudioTrackMultiple);

          let codec = match track.codec_id() {
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
          };

          let (_frames, size, _first) = frames
            .get(&track.track_number().into())
            .cloned()
            .unwrap_or_default();

          audio_track = Some(Track {
            codec,
            info: TrackInfo::Audio,
            size,
          });
        }
        TrackType::Video => {
          ensure!(video_track.is_none(), video_error::VideoTrackMultiple);

          let codec = match track.codec_id() {
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
          };

          let video = track
            .video()
            .context(video_error::VideoSettingsMissing { track: index })?;

          let (frames, size, first) = frames
            .get(&track.track_number().into())
            .cloned()
            .unwrap_or_default();

          let bit_depth = match (codec, first) {
            (Codec::Vp9, Some(first)) => {
              Some(Self::vp9_bit_depth(&first).context(video_error::Vp9FrameHeaderInvalid)?)
            }
            (Codec::Vp9, None) => None,
            _ => Some(8),
          };

          video_track = Some(Track {
            codec,
            info: TrackInfo::Video {
              bit_depth,
              dimensions: Dimensions {
                height: video.pixel_height().get(),
                width: video.pixel_width().get(),
              },
              frames,
            },
            size,
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

    let video_track = video_track.context(video_error::VideoTrackMissing)?;

    let mut tracks = vec![video_track];

    if let Some(audio_track) = audio_track {
      tracks.push(audio_track);
    }

    Ok(VideoInfo { duration, tracks })
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let info = self.info(root)?;

    self.duration = info.duration;
    self.tracks = info.tracks;

    Ok(())
  }

  pub(crate) fn resolutions(videos: &[Video]) -> Option<Resolutions> {
    Resolutions::new(videos.iter().filter_map(Video::dimensions), true)
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }

  fn vp9_bit_depth(data: &[u8]) -> Option<u64> {
    let mut reader = BitReader::new(data);

    // frame_marker
    if reader.bits(2)? != 2 {
      return None;
    }

    let profile_low_bit = reader.bit()?;
    let profile_high_bit = reader.bit()?;
    let profile = profile_high_bit << 1 | profile_low_bit;

    // reserved_zero
    if profile == 3 && reader.bit()? != 0 {
      return None;
    }

    // show_existing_frame
    if reader.bit()? != 0 {
      return None;
    }

    // frame_type
    if reader.bit()? != 0 {
      return None;
    }

    // show_frame
    reader.bits(1)?;

    // error_resilient_mode
    reader.bits(1)?;

    // frame_sync_code
    if reader.bits(24)? != 0x0049_8342 {
      return None;
    }

    let bit_depth = if profile >= 2 {
      // ten_or_twelve_bit
      if reader.bit()? == 0 { 10 } else { 12 }
    } else {
      8
    };

    Some(bit_depth)
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
  fn dimensions() {
    let mut foo = "foo.mp4".parse::<Video>().unwrap();

    assert_eq!(foo.dimensions(), None);

    foo.tracks = vec![
      Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio,
        size: 0,
      },
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          bit_depth: Some(8),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
        },
        size: 0,
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
      "a400000167666f6f2e6d703402800300",
    );

    assert_cbor(
      Video {
        duration: 3,
        filename: "foo.mp4".parse().unwrap(),
        tracks: vec![
          Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              bit_depth: Some(8),
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              frames: 0,
            },
            size: 0,
          },
          Track {
            codec: Codec::Mp3,
            info: TrackInfo::Audio,
            size: 0,
          },
        ],
        ty: VideoType::Mp4,
      },
      "a400030167666f6f2e6d70340282a30001018201a3000801a20001010202000200a30002010002000300",
    );
  }

  #[test]
  fn formats() {
    let mut foo = "foo.mp4".parse::<Video>().unwrap();

    foo.tracks = vec![
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          bit_depth: Some(8),
          dimensions: Dimensions::default(),
          frames: 0,
        },
        size: 0,
      },
      Track {
        codec: Codec::Aac,
        info: TrackInfo::Audio,
        size: 0,
      },
    ];

    let mut bar = foo.clone();
    bar.tracks[1].codec = Codec::Mp3;

    let mut baz = foo.clone();

    baz.tracks[0].info = TrackInfo::Video {
      bit_depth: Some(8),
      dimensions: Dimensions {
        height: 1,
        width: 2,
      },
      frames: 0,
    };

    let mut bob = "bob.webm".parse::<Video>().unwrap();

    bob.tracks = vec![
      Track {
        codec: Codec::Vp9,
        info: TrackInfo::Video {
          bit_depth: Some(8),
          dimensions: Dimensions::default(),
          frames: 0,
        },
        size: 0,
      },
      Track {
        codec: Codec::Opus,
        info: TrackInfo::Audio,
        size: 0,
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
  fn h264_bit_depth() {
    #[track_caller]
    fn case(sps: &[u8], expected: Option<u64>) {
      assert_eq!(Video::h264_bit_depth(sps), expected);
    }

    case(&[0x67, 66, 0, 30, 0x80], Some(8));
    case(&[0x67, 100, 0, 31, 0xA6], Some(10));
    case(&[0x67, 100, 0, 31, 0x91], Some(8));
    case(&[0x67, 100, 0, 0, 0x03, 0xA6], Some(10));
    case(&[0x67, 100, 0, 31], None);
    case(&[0x67], None);
    case(&[], None);
  }

  #[test]
  fn mp4_info() {
    #[track_caller]
    fn case(builder: Mp4Builder) -> Result<VideoInfo, VideoError> {
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
      VideoInfo {
        duration: 0,
        tracks: vec![
          Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              bit_depth: Some(8),
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              frames: 0,
            },
            size: 0,
          },
          Track {
            codec: Codec::Aac,
            info: TrackInfo::Audio,
            size: 0,
          },
        ],
      },
    );

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1)).unwrap(),
      VideoInfo {
        duration: 0,
        tracks: vec![Track {
          codec: Codec::H264,
          info: TrackInfo::Video {
            bit_depth: Some(8),
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            frames: 0,
          },
          size: 0,
        }],
      },
    );

    assert_eq!(
      case(
        Mp4Builder::new()
          .timescale(90000)
          .duration(45000)
          .video_track(2, 1),
      )
      .unwrap()
      .duration,
      500,
    );

    assert_eq!(
      case(Mp4Builder::new().timescale(3).duration(1).video_track(2, 1))
        .unwrap()
        .duration,
      333,
    );

    assert_eq!(
      case(Mp4Builder::new().frame_count(3).video_track(2, 1))
        .unwrap()
        .tracks[0]
        .info,
      TrackInfo::Video {
        bit_depth: Some(8),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 3,
      },
    );

    assert_eq!(
      case(
        Mp4Builder::new()
          .frame_count(3)
          .sample_size(5)
          .video_track(2, 1),
      )
      .unwrap()
      .tracks[0]
        .size,
      15,
    );

    assert_eq!(
      case(Mp4Builder::new().sample_sizes(&[3, 5]).video_track(2, 1))
        .unwrap()
        .tracks[0],
      Track {
        codec: Codec::H264,
        info: TrackInfo::Video {
          bit_depth: Some(8),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 2,
        },
        size: 8,
      },
    );

    assert_eq!(
      case(
        Mp4Builder::new()
          .sps(&[0x67, 100, 0, 31, 0xA6])
          .video_track(2, 1),
      )
      .unwrap()
      .tracks[0]
        .info,
      TrackInfo::Video {
        bit_depth: Some(10),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
      },
    );

    error(
      Mp4Builder::new().sps(&[0x67, 100, 0, 31]).video_track(2, 1),
      "invalid SPS",
    );

    error(
      Mp4Builder::new().avcc_profile(100).video_track(2, 1),
      "missing SPS",
    );

    error(
      Mp4Builder::new().timescale(0).video_track(2, 1),
      "zero timescale",
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
          &[Mp4Builder::video_entry(
            *b"s263",
            *b"d263",
            &[1, 0, 0, 0, 0xff, 0xe0, 0],
            2,
            1,
          )],
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
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              frames: 0,
            },
            size: 0,
          },
          Track {
            codec: Codec::Aac,
            info: TrackInfo::Audio,
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
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            frames: 0,
          },
          size: 0,
        },
        Track {
          codec: Codec::Mp3,
          info: TrackInfo::Audio,
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
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              frames: 0,
            },
            size: 0,
          },
          Track {
            codec: Codec::Mp3,
            info: TrackInfo::Audio,
            size: 0,
          },
        ],
        ty: VideoType::Mp4,
      })
      .unwrap(),
      r#"{"duration":0,"filename":"foo.mp4","tracks":[{"codec":"h264","info":{"type":"video","bit_depth":8,"dimensions":{"height":1,"width":2},"frames":0},"size":0},{"codec":"mp3","info":{"type":"audio"},"size":0}],"type":"mp4"}"#,
    );
  }

  #[test]
  fn vp9_bit_depth() {
    #[track_caller]
    fn case(data: &[u8], expected: Option<u64>) {
      assert_eq!(Video::vp9_bit_depth(data), expected,);
    }

    case(&[0x82, 0x49, 0x83, 0x42], Some(8));
    case(&[0x92, 0x49, 0x83, 0x42, 0x00], Some(10));
    case(&[0x92, 0x49, 0x83, 0x42, 0x80], Some(12));
    case(&[0x84, 0x49, 0x83, 0x42], None);
    case(&[0x88, 0x49, 0x83, 0x42], None);
    case(&[0x82, 0x49, 0x83, 0x43], None);
    case(&[0x82], None);
    case(b"foo", None);
    case(&[], None);
  }

  #[test]
  fn webm_info() {
    #[track_caller]
    fn case(builder: WebmBuilder) -> Result<VideoInfo, VideoError> {
      Video::info_webm(io::Cursor::new(builder.build()))
    }

    #[track_caller]
    fn error(builder: WebmBuilder, expected: &str) {
      assert_eq!(case(builder).unwrap_err().to_string(), expected);
    }

    assert_eq!(
      case(WebmBuilder::new().video_track(2, 1).audio_track("A_OPUS")).unwrap(),
      VideoInfo {
        duration: 0,
        tracks: vec![
          Track {
            codec: Codec::Vp9,
            info: TrackInfo::Video {
              bit_depth: None,
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              frames: 0,
            },
            size: 0,
          },
          Track {
            codec: Codec::Opus,
            info: TrackInfo::Audio,
            size: 0,
          },
        ],
      },
    );

    assert_eq!(
      case(
        WebmBuilder::new()
          .track(1, "V_VP8", &WebmBuilder::video_settings(2, 1))
          .audio_track("A_VORBIS"),
      )
      .unwrap(),
      VideoInfo {
        duration: 0,
        tracks: vec![
          Track {
            codec: Codec::Vp8,
            info: TrackInfo::Video {
              bit_depth: Some(8),
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              frames: 0,
            },
            size: 0,
          },
          Track {
            codec: Codec::Vorbis,
            info: TrackInfo::Audio,
            size: 0,
          },
        ],
      },
    );

    assert_eq!(
      case(WebmBuilder::new().video_track(2, 1)).unwrap(),
      VideoInfo {
        duration: 0,
        tracks: vec![Track {
          codec: Codec::Vp9,
          info: TrackInfo::Video {
            bit_depth: None,
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            frames: 0,
          },
          size: 0,
        }],
      },
    );

    assert_eq!(
      case(WebmBuilder::new().duration(1500.0).video_track(2, 1))
        .unwrap()
        .duration,
      1500,
    );

    assert_eq!(
      case(
        WebmBuilder::new()
          .timestamp_scale(2_000_000)
          .duration(3.0)
          .video_track(2, 1),
      )
      .unwrap()
      .duration,
      6,
    );

    assert_eq!(
      case(
        WebmBuilder::new()
          .video_track(2, 1)
          .frame(1, &[0x82, 0x49, 0x83, 0x42])
          .frame(1, b"")
      )
      .unwrap()
      .tracks[0]
        .info,
      TrackInfo::Video {
        bit_depth: Some(8),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 2,
      },
    );

    assert_eq!(
      case(
        WebmBuilder::new()
          .video_track(2, 1)
          .audio_track("A_OPUS")
          .frame(1, &[0x82, 0x49, 0x83, 0x42])
          .frame(2, b"ab"),
      )
      .unwrap()
      .tracks,
      vec![
        Track {
          codec: Codec::Vp9,
          info: TrackInfo::Video {
            bit_depth: Some(8),
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            frames: 1,
          },
          size: 4,
        },
        Track {
          codec: Codec::Opus,
          info: TrackInfo::Audio,
          size: 2,
        },
      ],
    );

    assert_eq!(
      case(
        WebmBuilder::new()
          .video_track(2, 1)
          .frame(1, &[0x92, 0x49, 0x83, 0x42, 0x00]),
      )
      .unwrap()
      .tracks[0]
        .info,
      TrackInfo::Video {
        bit_depth: Some(10),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 1,
      },
    );

    error(
      WebmBuilder::new().video_track(2, 1).frame(1, b"foo"),
      "invalid VP9 frame header",
    );

    error(
      WebmBuilder::new().no_duration().video_track(2, 1),
      "missing duration",
    );

    error(
      WebmBuilder::new().duration(f64::NAN).video_track(2, 1),
      "invalid duration",
    );

    error(
      WebmBuilder::new().duration(1e22).video_track(2, 1),
      "duration overflow",
    );

    error(
      WebmBuilder::new()
        .timestamp_scale(u64::from(u32::MAX) + 1)
        .video_track(2, 1),
      "unsupported timestamp scale 4294967296",
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
