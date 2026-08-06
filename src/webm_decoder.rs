use super::*;

pub(crate) struct WebmDecoder;

impl WebmDecoder {
  fn decode<T: Read + Seek>(reader: T) -> Result<VideoMetadata, VideoError> {
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

          let audio = track
            .audio()
            .context(video_error::AudioSettingsMissing { track: index })?;

          let sample_rate =
            audio
              .sampling_frequency()
              .into_u64()
              .context(video_error::SampleRateInvalid {
                sample_rate: audio.sampling_frequency(),
                track: index,
              })?;

          let (_frames, size, _first) = frames
            .get(&track.track_number().into())
            .cloned()
            .unwrap_or_default();

          audio_track = Some(Track {
            codec,
            info: TrackInfo::Audio {
              channels: audio.channels().get(),
              sample_rate,
            },
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

          let color_info = match (codec, first) {
            (Codec::Vp9, Some(first)) => {
              Some(Self::vp9_color_info(&first).context(video_error::Vp9FrameHeaderInvalid)?)
            }
            (Codec::Vp9, None) => None,
            _ => Some(ColorInfo {
              bit_depth: 8,
              chroma_subsampling: ChromaSubsampling::Yuv420,
            }),
          };

          video_track = Some(Track {
            codec,
            info: TrackInfo::Video {
              bit_depth: color_info.map(|color_info| color_info.bit_depth),
              chroma_subsampling: color_info.map(|color_info| color_info.chroma_subsampling),
              dimensions: Dimensions {
                height: video.pixel_height().get(),
                width: video.pixel_width().get(),
              },
              frames,
              orientation: Orientation::new(),
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

    Ok(VideoMetadata { duration, tracks })
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<VideoMetadata> {
    let file = filesystem::open(path)?;

    Self::decode(file).context(error::Video { path })
  }

  fn vp9_color_info(data: &[u8]) -> Option<ColorInfo> {
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

    // color_space
    let color_space = reader.bits(3)?;

    let chroma_subsampling = if color_space == 7 {
      if profile == 1 || profile == 3 {
        // reserved_zero
        if reader.bit()? != 0 {
          return None;
        }

        ChromaSubsampling::Yuv444
      } else {
        return None;
      }
    } else {
      // color_range
      reader.bit()?;

      if profile == 1 || profile == 3 {
        let subsampling_x = reader.bit()?;
        let subsampling_y = reader.bit()?;

        // reserved_zero
        if reader.bit()? != 0 {
          return None;
        }

        match (subsampling_x, subsampling_y) {
          (0, 0) => ChromaSubsampling::Yuv444,
          (0, 1) => ChromaSubsampling::Yuv440,
          (1, 0) => ChromaSubsampling::Yuv422,
          _ => return None,
        }
      } else {
        ChromaSubsampling::Yuv420
      }
    };

    Some(ColorInfo {
      bit_depth,
      chroma_subsampling,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode() {
    #[track_caller]
    fn case(builder: WebmBuilder) -> Result<VideoMetadata, VideoError> {
      WebmDecoder::decode(io::Cursor::new(builder.build()))
    }

    #[track_caller]
    fn error(builder: WebmBuilder, expected: &str) {
      assert_eq!(case(builder).unwrap_err().to_string(), expected);
    }

    assert_eq!(
      case(WebmBuilder::new().video_track(2, 1).audio_track("A_OPUS")).unwrap(),
      VideoMetadata {
        duration: 0,
        tracks: vec![
          Track {
            codec: Codec::Vp9,
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
          },
          Track {
            codec: Codec::Opus,
            info: TrackInfo::Audio {
              channels: 2,
              sample_rate: 44100,
            },
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
      VideoMetadata {
        duration: 0,
        tracks: vec![
          Track {
            codec: Codec::Vp8,
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
            codec: Codec::Vorbis,
            info: TrackInfo::Audio {
              channels: 2,
              sample_rate: 44100,
            },
            size: 0,
          },
        ],
      },
    );

    assert_eq!(
      case(WebmBuilder::new().video_track(2, 1)).unwrap(),
      VideoMetadata {
        duration: 0,
        tracks: vec![Track {
          codec: Codec::Vp9,
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
          .frame(1, &[0x82, 0x49, 0x83, 0x42, 0x00])
          .frame(1, b"")
      )
      .unwrap()
      .tracks[0]
        .info,
      TrackInfo::Video {
        bit_depth: Some(8),
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 2,
        orientation: Orientation::new(),
      },
    );

    assert_eq!(
      case(
        WebmBuilder::new()
          .video_track(2, 1)
          .audio_track("A_OPUS")
          .frame(1, &[0x82, 0x49, 0x83, 0x42, 0x00])
          .frame(2, b"ab"),
      )
      .unwrap()
      .tracks,
      vec![
        Track {
          codec: Codec::Vp9,
          info: TrackInfo::Video {
            bit_depth: Some(8),
            chroma_subsampling: Some(ChromaSubsampling::Yuv420),
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            frames: 1,
            orientation: Orientation::new(),
          },
          size: 5,
        },
        Track {
          codec: Codec::Opus,
          info: TrackInfo::Audio {
            channels: 2,
            sample_rate: 44100,
          },
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
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 1,
        orientation: Orientation::new(),
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
      WebmBuilder::new().video_track(2, 1).track(2, "A_OPUS", &[]),
      "track 1 has missing audio settings",
    );
    error(
      WebmBuilder::new()
        .video_track(2, 1)
        .track(2, "A_OPUS", &WebmBuilder::audio_settings(2, 0.5)),
      "track 1 has invalid sample rate 0.5",
    );
    error(
      WebmBuilder::new().video_track(2, 1).doc_type("matroska"),
      "expected DocType `webm` but found `matroska`",
    );

    assert_eq!(
      WebmDecoder::decode(io::Cursor::new(b"foo"))
        .unwrap_err()
        .to_string(),
      "failed to decode WebM",
    );
  }

  #[test]
  fn vp9_color_info() {
    #[track_caller]
    fn case(data: &[u8], expected: Option<ColorInfo>) {
      assert_eq!(WebmDecoder::vp9_color_info(data), expected);
    }

    fn config(bit_depth: u64, chroma_subsampling: ChromaSubsampling) -> ColorInfo {
      ColorInfo {
        bit_depth,
        chroma_subsampling,
      }
    }

    case(
      &[0x82, 0x49, 0x83, 0x42, 0x00],
      Some(config(8, ChromaSubsampling::Yuv420)),
    );
    case(
      &[0x92, 0x49, 0x83, 0x42, 0x00],
      Some(config(10, ChromaSubsampling::Yuv420)),
    );
    case(
      &[0x92, 0x49, 0x83, 0x42, 0x80],
      Some(config(12, ChromaSubsampling::Yuv420)),
    );
    case(
      &[0xA2, 0x49, 0x83, 0x42, 0x08],
      Some(config(8, ChromaSubsampling::Yuv422)),
    );
    case(
      &[0xA2, 0x49, 0x83, 0x42, 0x04],
      Some(config(8, ChromaSubsampling::Yuv440)),
    );
    case(
      &[0xA2, 0x49, 0x83, 0x42, 0x00],
      Some(config(8, ChromaSubsampling::Yuv444)),
    );
    case(
      &[0xA2, 0x49, 0x83, 0x42, 0xE0],
      Some(config(8, ChromaSubsampling::Yuv444)),
    );
    case(&[0xA2, 0x49, 0x83, 0x42, 0x0C], None);
    case(&[0xA2, 0x49, 0x83, 0x42, 0x0A], None);
    case(&[0x82, 0x49, 0x83, 0x42, 0xE0], None);
    case(&[0x82, 0x49, 0x83, 0x42], None);
    case(&[0x84, 0x49, 0x83, 0x42], None);
    case(&[0x88, 0x49, 0x83, 0x42], None);
    case(&[0x82, 0x49, 0x83, 0x43], None);
    case(&[0x82], None);
    case(b"foo", None);
    case(&[], None);
  }
}
