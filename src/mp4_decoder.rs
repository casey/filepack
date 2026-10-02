use {
  super::*,
  mp4::{Fourcc, Moov, Mp4, Mp4a, SampleEntry, Tkhd},
};

pub(crate) struct Mp4Decoder;

impl Mp4Decoder {
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

  fn metadata<T: Read + Seek>(reader: T, size: u64) -> Result<VideoMetadata, VideoError> {
    fn codec(mp4a: &Mp4a) -> Option<TrackCodec> {
      match mp4a.esds.object_type {
        0x40 | 0x66 | 0x67 => Some(TrackCodec::Aac),
        0x69 | 0x6b => Some(TrackCodec::Mp3),
        _ => None,
      }
    }

    fn codec_name(entry: Option<&SampleEntry>) -> String {
      match entry {
        Some(SampleEntry::Avc1(_)) => "H.264".into(),
        Some(SampleEntry::Mp4a(mp4a)) => match codec(mp4a) {
          Some(codec) => codec.to_string(),
          None => "unknown".into(),
        },
        Some(SampleEntry::Unknown(ty)) => match &ty.0 {
          b"av01" => "AV1".into(),
          b"hev1" | b"hvc1" => "H.265".into(),
          b"tx3g" => "TTXT".into(),
          b"vp08" => "VP8".into(),
          b"vp09" => "VP9".into(),
          _ => ty.to_string(),
        },
        None => "unknown".into(),
      }
    }

    fn orientation(tkhd: &Tkhd) -> Option<Orientation> {
      const U: i32 = 0x0001_0000;
      const N: i32 = -0x0001_0000;

      let [a, b, _, c, d, ..] = tkhd.matrix;

      let (mirrored, rotation) = match (a, b, c, d) {
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

    let mp4 = Mp4::read(BufReader::new(reader), size).context(video_error::DecodeMp4)?;

    let mvhd = &mp4.moov.mvhd;

    ensure!(mvhd.timescale != 0, video_error::TimescaleZero);

    let duration = u64::try_from(u128::from(mvhd.duration) * 1000 / u128::from(mvhd.timescale))
      .ok()
      .context(video_error::DurationOverflow)?;

    let mut video_track = None;
    let mut audio_track = None;

    for (index, trak) in mp4.moov.traks.iter().enumerate() {
      let stbl = &trak.mdia.minf.stbl;

      let entry = stbl.stsd.entries.first();

      let size = stbl.stsz.size();

      match &trak.mdia.hdlr.handler_type.0 {
        b"soun" => {
          ensure!(audio_track.is_none(), video_error::AudioTrackMultiple);

          let Some(SampleEntry::Mp4a(mp4a)) = entry else {
            return Err(VideoError::AudioCodecUnsupported {
              codec: codec_name(entry),
              track: index,
            });
          };

          let Some(codec) = codec(mp4a) else {
            return Err(VideoError::AudioCodecUnsupported {
              codec: codec_name(entry),
              track: index,
            });
          };

          audio_track = Some(Track {
            codec: Some(codec),
            info: Some(TrackInfo::Audio {
              channels: mp4a.channels.into(),
              sample_rate: mp4a.sample_rate.into(),
            }),
            size,
          });
        }
        b"vide" => {
          ensure!(video_track.is_none(), video_error::VideoTrackMultiple);

          let Some(SampleEntry::Avc1(avc1)) = entry else {
            return Err(VideoError::VideoCodecUnsupported {
              codec: codec_name(entry),
              track: index,
            });
          };

          let color_info = if let Some(sps) = avc1.avcc.sequence_parameter_sets.first() {
            Self::h264_color_info(sps).context(video_error::SpsInvalid)?
          } else {
            ensure!(
              !Self::h264_high_profile(avc1.avcc.profile.into()),
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
            codec: Some(TrackCodec::H264),
            info: Some(TrackInfo::Video {
              bit_depth: color_info.bit_depth,
              chroma_subsampling: Some(color_info.chroma_subsampling),
              dimensions: Dimensions {
                height: avc1.height.into(),
                width: avc1.width.into(),
              },
              frames: stbl.stsz.sample_count.into(),
              orientation,
            }),
            size,
          });
        }
        _ => {
          return Err(
            video_error::TrackUnsupported {
              track: index,
              ty: trak.mdia.hdlr.name(),
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

    let title = Self::title(&mp4.moov)?;

    Ok(VideoMetadata {
      duration,
      title,
      tracks,
    })
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<VideoMetadata> {
    let file = filesystem::open(path)?;

    let size = file
      .metadata()
      .context(filesystem_error::Io { path })?
      .len();

    Self::metadata(file, size).context(error::Video { path })
  }

  fn title(moov: &Moov) -> Result<Option<Text>, VideoError> {
    let Some(ilst) = moov.ilst() else {
      return Ok(None);
    };

    let tag = "©nam";

    let titles = ilst
      .text(Fourcc(*b"\xa9nam"))
      .context(video_error::DecodeMp4)?;

    ensure!(titles.len() <= 1, video_error::TagMultiple { tag });

    let Some(title) = titles.first() else {
      return Ok(None);
    };

    Ok(Some(
      title
        .parse::<Text>()
        .context(video_error::TagInvalid { tag })?,
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
      &[0x67, 100, 0, 31, 0xa6],
      Some(config(10, ChromaSubsampling::Yuv420)),
    );
    case(
      &[0x67, 100, 0, 31, 0xb8],
      Some(config(8, ChromaSubsampling::Yuv422)),
    );
    case(
      &[0x67, 100, 0, 31, 0x91],
      Some(config(8, ChromaSubsampling::Yuv444)),
    );
    case(
      &[0x67, 100, 0, 31, 0xe0],
      Some(config(8, ChromaSubsampling::Yuv400)),
    );
    case(
      &[0x67, 100, 0, 0, 0x03, 0xa6],
      Some(config(10, ChromaSubsampling::Yuv420)),
    );
    case(&[0x67, 100, 0, 31, 0x94], None);
    case(&[0x67, 100, 0, 31], None);
    case(&[0x67], None);
    case(&[], None);
  }

  #[test]
  fn metadata() {
    const U: i32 = 0x0001_0000;
    const N: i32 = -0x0001_0000;

    #[track_caller]
    fn case(builder: Mp4Builder) -> Result<VideoMetadata, VideoError> {
      let bytes = builder.build();
      let size = bytes.len().try_into().unwrap();
      Mp4Decoder::metadata(io::Cursor::new(bytes), size)
    }

    #[track_caller]
    fn error(builder: Mp4Builder, expected: &str) {
      assert_eq!(
        case(builder)
          .unwrap_err()
          .iter_chain()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(": "),
        expected,
      );
    }

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1).audio_track(0x40)).unwrap(),
      VideoMetadata {
        duration: 0,
        title: None,
        tracks: vec![
          Track {
            codec: Some(TrackCodec::H264),
            info: Some(TrackInfo::Video {
              bit_depth: 8,
              chroma_subsampling: Some(ChromaSubsampling::Yuv420),
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              frames: 0,
              orientation: Orientation::new(),
            }),
            size: 0,
          },
          Track {
            codec: Some(TrackCodec::Aac),
            info: Some(TrackInfo::Audio {
              channels: 2,
              sample_rate: 44100,
            }),
            size: 0,
          },
        ],
      },
    );

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1)).unwrap(),
      VideoMetadata {
        duration: 0,
        title: None,
        tracks: vec![Track {
          codec: Some(TrackCodec::H264),
          info: Some(TrackInfo::Video {
            bit_depth: 8,
            chroma_subsampling: Some(ChromaSubsampling::Yuv420),
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            frames: 0,
            orientation: Orientation::new(),
          }),
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
      Some(TrackInfo::Video {
        bit_depth: 8,
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 3,
        orientation: Orientation::new(),
      }),
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
        codec: Some(TrackCodec::H264),
        info: Some(TrackInfo::Video {
          bit_depth: 8,
          chroma_subsampling: Some(ChromaSubsampling::Yuv420),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 2,
          orientation: Orientation::new(),
        }),
        size: 8,
      },
    );

    assert_eq!(
      case(
        Mp4Builder::new()
          .sps(&[0x67, 100, 0, 31, 0xa6])
          .video_track(2, 1),
      )
      .unwrap()
      .tracks[0]
        .info,
      Some(TrackInfo::Video {
        bit_depth: 10,
        chroma_subsampling: Some(ChromaSubsampling::Yuv420),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
        orientation: Orientation::new(),
      }),
    );

    for (matrix, mirrored, rotation) in [
      ([U, 0, 0, U], false, Rotation::R0),
      ([N, 0, 0, U], true, Rotation::R0),
      ([0, U, N, 0], false, Rotation::R90),
      ([0, U, U, 0], true, Rotation::R90),
      ([N, 0, 0, N], false, Rotation::R180),
      ([U, 0, 0, N], true, Rotation::R180),
      ([0, N, U, 0], false, Rotation::R270),
      ([0, N, N, 0], true, Rotation::R270),
    ] {
      let [a, b, c, d] = matrix;

      assert_eq!(
        case(
          Mp4Builder::new()
            .matrix([a, b, 0, c, d, 0, 0, 0, 0x4000_0000])
            .video_track(2, 1),
        )
        .unwrap()
        .tracks[0]
          .info,
        Some(TrackInfo::Video {
          bit_depth: 8,
          chroma_subsampling: Some(ChromaSubsampling::Yuv420),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          frames: 0,
          orientation: Orientation { mirrored, rotation },
        }),
      );
    }

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
        .track(*b"meta", 1000, &[]),
      "track 2 has unsupported track type `metadata`",
    );
    error(
      Mp4Builder::new()
        .track(
          *b"vide",
          1000,
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

    for (fourcc, name) in [
      (*b"av01", "AV1"),
      (*b"hev1", "H.265"),
      (*b"hvc1", "H.265"),
      (*b"tx3g", "TTXT"),
      (*b"vp08", "VP8"),
      (*b"vp09", "VP9"),
    ] {
      error(
        Mp4Builder::new().track(
          *b"vide",
          1000,
          &[Mp4Builder::video_entry(fourcc, *b"dfLa", &[], 2, 1)],
        ),
        &format!("track 0 has unsupported video codec `{name}`"),
      );
    }

    let builder = Mp4Builder::new();
    let entry = builder.audio_entry(0x40);

    error(
      builder.track(*b"vide", 1000, &[entry]),
      "track 0 has unsupported video codec `AAC`",
    );

    error(
      Mp4Builder::new().track(*b"vide", 1000, &[]),
      "track 0 has unsupported video codec `unknown`",
    );

    error(
      Mp4Builder::new().video_track(2, 1).track(
        *b"soun",
        44100,
        &[Mp4Builder::video_entry(
          *b"avc1",
          *b"avcC",
          &[1, 0, 0, 0, 0xff, 0xe0, 0],
          2,
          1,
        )],
      ),
      "track 1 has unsupported audio codec `H.264`",
    );
    error(
      Mp4Builder::new().video_track(2, 1).audio_track(0x11),
      "track 1 has unsupported audio codec `unknown`",
    );

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1).name("foo"))
        .unwrap()
        .title,
      Some("foo".parse().unwrap()),
    );

    assert_eq!(
      case(Mp4Builder::new().video_track(2, 1).tag(*b"\xa9ART", "foo"))
        .unwrap()
        .title,
      None,
    );

    error(
      Mp4Builder::new().video_track(2, 1).name("foo").name("bar"),
      "multiple `©nam` tags",
    );

    error(
      Mp4Builder::new().video_track(2, 1).name(b""),
      "invalid `©nam` tag: text may not be empty",
    );

    error(
      Mp4Builder::new().video_track(2, 1).name(b"\xff"),
      "failed to decode MP4: `©nam` tag is not valid UTF-8: invalid utf-8 sequence of 1 bytes from index 0",
    );

    error(
      Mp4Builder::new().video_track(2, 1).name("\0"),
      "invalid `©nam` tag: text may not contain control character `\\u{0}`",
    );

    assert_eq!(
      Mp4Decoder::metadata(io::Cursor::new(b"foo"), 3)
        .unwrap_err()
        .iter_chain()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(": "),
      "failed to decode MP4: truncated",
    );
  }
}
