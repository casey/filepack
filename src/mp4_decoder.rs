use super::*;

pub(crate) struct Mp4Decoder;

impl Mp4Decoder {
  fn decode<T: Read + Seek>(reader: T, size: u64) -> Result<VideoMetadata, VideoError> {
    use re_mp4::{Mp4, Mp4aBox, StsdBoxContent, TkhdBox};

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

    fn orientation(tkhd: &TkhdBox) -> Option<Orientation> {
      const U: i32 = 0x0001_0000;
      const N: i32 = -0x0001_0000;

      let matrix = &tkhd.matrix;

      let (mirrored, rotation) = match (matrix.a, matrix.b, matrix.c, matrix.d) {
        (U, 0, 0, U) => (false, Rotation::R0),
        (N, 0, 0, U) => (true, Rotation::R0),
        (0, U, N, 0) => (false, Rotation::R90),
        (0, U, U, 0) => (true, Rotation::R90),
        (N, 0, 0, N) => (false, Rotation::R180),
        (U, 0, 0, N) => (true, Rotation::R180),
        (0, N, U, 0) => (false, Rotation::R270),
        (0, N, N, 0) => (true, Rotation::R270),
        _ => return None,
      };

      Some(Orientation { mirrored, rotation })
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

          let StsdBoxContent::Mp4a(mp4a) = contents else {
            return Err(
              video_error::AudioCodecUnsupported {
                codec: codec_name(contents),
                track: index,
              }
              .build(),
            );
          };

          let Some(codec) = mp4a_codec(mp4a) else {
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
            info: TrackInfo::Audio {
              channels: mp4a.channelcount.into(),
              sample_rate: mp4a.samplerate.value().into(),
            },
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

          let color_info = if let Some(sps) = avc1.avcc.sequence_parameter_sets.first() {
            Self::h264_color_info(&sps.bytes).context(video_error::SpsInvalid)?
          } else {
            ensure!(
              !Self::h264_high_profile(avc1.avcc.avc_profile_indication.into()),
              video_error::SpsMissing,
            );

            ColorInfo {
              bit_depth: 8,
              chroma_subsampling: ChromaSubsampling::Yuv420,
            }
          };

          let orientation =
            orientation(&trak.tkhd).context(video_error::MatrixUnsupported { track: index })?;

          video_track = Some(Track {
            codec: Codec::H264,
            info: TrackInfo::Video {
              bit_depth: Some(color_info.bit_depth),
              chroma_subsampling: Some(color_info.chroma_subsampling),
              dimensions: Dimensions {
                height: avc1.height.into(),
                width: avc1.width.into(),
              },
              frames: stsz.sample_count.into(),
              orientation,
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

    Ok(VideoMetadata { duration, tracks })
  }

  fn h264_color_info(sps: &[u8]) -> Option<ColorInfo> {
    let mut rbsp = Vec::new();

    // skip NAL unit header
    for &byte in sps.get(1..)? {
      // remove emulation prevention bytes
      if byte == 3 && rbsp.ends_with(&[0, 0]) {
        continue;
      }

      rbsp.push(byte);
    }

    let mut reader = BitReader::new(&rbsp);

    // profile_idc
    let profile_idc = reader.bits(8)?;

    // constraint flags
    reader.bits(8)?;

    // level_idc
    reader.bits(8)?;

    // seq_parameter_set_id
    reader.ue()?;

    if !Self::h264_high_profile(profile_idc) {
      return Some(ColorInfo {
        bit_depth: 8,
        chroma_subsampling: ChromaSubsampling::Yuv420,
      });
    }

    // chroma_format_idc
    let chroma_subsampling = match reader.ue()? {
      0 => ChromaSubsampling::Yuv400,
      1 => ChromaSubsampling::Yuv420,
      2 => ChromaSubsampling::Yuv422,
      3 => {
        // separate_colour_plane_flag
        reader.bit()?;
        ChromaSubsampling::Yuv444
      }
      _ => return None,
    };

    // bit_depth_luma_minus8
    let bit_depth = 8 + reader.ue()?;

    Some(ColorInfo {
      bit_depth,
      chroma_subsampling,
    })
  }

  fn h264_high_profile(profile_idc: u64) -> bool {
    matches!(
      profile_idc,
      44 | 83 | 86 | 100 | 110 | 118 | 122 | 128 | 134 | 135 | 138 | 139 | 244
    )
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<VideoMetadata> {
    let file = filesystem::open(path)?;

    let size = file.metadata().context(error::FilesystemIo { path })?.len();

    Self::decode(file, size).context(error::Video { path })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode() {
    #[track_caller]
    fn case(builder: Mp4Builder) -> Result<VideoMetadata, VideoError> {
      let bytes = builder.build();
      let size = bytes.len().try_into().unwrap();
      Mp4Decoder::decode(io::Cursor::new(bytes), size)
    }

    #[track_caller]
    fn error(builder: Mp4Builder, expected: &str) {
      assert_eq!(case(builder).unwrap_err().to_string(), expected);
    }

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1).audio_track(0x40)).unwrap(),
      VideoMetadata {
        duration: 0,
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
      },
    );

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1)).unwrap(),
      VideoMetadata {
        duration: 0,
        tracks: vec![Track {
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
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 3,
        orientation: Orientation::new(),
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
          chroma_subsampling: Some(ChromaSubsampling::Yuv420),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 2,
          orientation: Orientation::new(),
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
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
        orientation: Orientation::new(),
      },
    );

    assert_eq!(
      case(
        Mp4Builder::new()
          .matrix([0, 0x0001_0000, 0, -0x0001_0000, 0, 0, 0, 0, 0x4000_0000])
          .video_track(2, 1),
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
        frames: 0,
        orientation: Orientation {
          mirrored: false,
          rotation: Rotation::R90,
        },
      },
    );

    assert_eq!(
      case(
        Mp4Builder::new()
          .matrix([-0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000])
          .video_track(2, 1),
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
        frames: 0,
        orientation: Orientation {
          mirrored: true,
          rotation: Rotation::R0,
        },
      },
    );

    error(
      Mp4Builder::new().matrix([0; 9]).video_track(2, 1),
      "track 0 has unsupported transformation matrix",
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
      Mp4Decoder::decode(io::Cursor::new(b"foo"), 3)
        .unwrap_err()
        .to_string(),
      "failed to decode MP4",
    );
  }

  #[test]
  fn h264_color_info() {
    #[track_caller]
    fn case(sps: &[u8], expected: Option<ColorInfo>) {
      assert_eq!(Mp4Decoder::h264_color_info(sps), expected);
    }

    fn config(bit_depth: u64, chroma_subsampling: ChromaSubsampling) -> ColorInfo {
      ColorInfo {
        bit_depth,
        chroma_subsampling,
      }
    }

    case(
      &[0x67, 66, 0, 30, 0x80],
      Some(config(8, ChromaSubsampling::Yuv420)),
    );
    case(
      &[0x67, 100, 0, 31, 0xA6],
      Some(config(10, ChromaSubsampling::Yuv420)),
    );
    case(
      &[0x67, 100, 0, 31, 0xB8],
      Some(config(8, ChromaSubsampling::Yuv422)),
    );
    case(
      &[0x67, 100, 0, 31, 0x91],
      Some(config(8, ChromaSubsampling::Yuv444)),
    );
    case(
      &[0x67, 100, 0, 31, 0xE0],
      Some(config(8, ChromaSubsampling::Yuv400)),
    );
    case(
      &[0x67, 100, 0, 0, 0x03, 0xA6],
      Some(config(10, ChromaSubsampling::Yuv420)),
    );
    case(&[0x67, 100, 0, 31], None);
    case(&[0x67], None);
    case(&[], None);
  }
}
