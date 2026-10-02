use {
  super::*,
  mp4::{Fourcc, Ilst, Moov, Mp4, SampleEntry, Trak},
};

const ALBUM: Fourcc = Fourcc(*b"\xa9alb");
const ARTIST: Fourcc = Fourcc(*b"\xa9ART");
const COVER: Fourcc = Fourcc(*b"covr");
const DISC: Fourcc = Fourcc(*b"disk");
const TITLE: Fourcc = Fourcc(*b"\xa9nam");
const TRACK: Fourcc = Fourcc(*b"trkn");

#[derive(Clone, Debug, PartialEq)]
struct AudioProperties {
  channels: u64,
  sample_rate: u64,
  samples: u64,
  size: u64,
}

pub(crate) struct M4aDecoder;

impl M4aDecoder {
  pub(crate) fn cover_art(data: &[u8]) -> Result<Vec<EmbeddedImage>, AudioError> {
    fn cover_art(data: &[u8]) -> Result<Vec<EmbeddedImage>, M4aError> {
      let mp4 = Mp4::parse(data)?;

      let Some(ilst) = mp4.moov.ilst() else {
        return Ok(Vec::new());
      };

      ilst
        .data(COVER)
        .map(|data| {
          let media_type = match data.data_type {
            13 => mime::IMAGE_JPEG,
            14 => mime::IMAGE_PNG,
            data_type => return Err(M4aError::CoverDataType { data_type }),
          };

          Ok(EmbeddedImage {
            data: data.payload.clone(),
            media_type,
          })
        })
        .collect()
    }

    cover_art(data).context(audio_error::M4aDecode)
  }

  fn metadata(data: &[u8]) -> Result<AudioMetadata, AudioError> {
    let mp4 = Mp4::parse(data)
      .map_err(M4aError::from)
      .context(audio_error::M4aDecode)?;

    let AudioProperties {
      channels,
      sample_rate,
      samples,
      size,
    } = Self::properties(&mp4.moov).context(audio_error::M4aDecode)?;

    let ilst = mp4.moov.ilst();

    let album = Self::text_tag(ilst, ALBUM, "©alb")?;
    let artist = Self::text_tag(ilst, ARTIST, "©ART")?;
    let (disc, discs) = Self::pair_tag(ilst, DISC, "disk")?;
    let discs = discs.context(audio_error::DiscTotalMissing { tag: "disk" })?;
    let title = Self::text_tag(ilst, TITLE, "©nam")?;
    let (track, tracks) = Self::pair_tag(ilst, TRACK, "trkn")?;
    let tracks = tracks.context(audio_error::TrackTotalMissing { tag: "trkn" })?;

    Ok(AudioMetadata {
      album,
      artist,
      channels,
      codec: AudioCodec::Aac,
      disc,
      discs,
      sample_bits: None,
      sample_rate,
      samples,
      size,
      title,
      track,
      tracks,
    })
  }

  fn pair_tag(
    ilst: Option<&Ilst>,
    fourcc: Fourcc,
    tag: &'static str,
  ) -> Result<(u64, Option<u64>), AudioError> {
    let mut data = ilst.into_iter().flat_map(|ilst| ilst.data(fourcc));

    let first = data.next().context(audio_error::TagMissing { tag })?;

    ensure!(data.next().is_none(), audio_error::TagMultiple { tag });

    let (number, total) = first
      .pair(fourcc)
      .map_err(M4aError::from)
      .context(audio_error::M4aDecode)?;

    Ok((number.into(), (total != 0).then_some(total.into())))
  }

  fn properties(moov: &Moov) -> Result<AudioProperties, M4aError> {
    let mut audio = None;

    for (index, trak) in moov.traks.iter().enumerate() {
      match &trak.mdia.hdlr.handler_type.0 {
        b"soun" => {
          ensure!(audio.is_none(), m4a_error::AudioTrackMultiple);
          audio = Some(Self::track(trak)?);
        }
        _ => {
          return Err(M4aError::TrackUnsupported {
            track: index,
            ty: trak.mdia.hdlr.name(),
          });
        }
      }
    }

    audio.context(m4a_error::AudioTrackMissing)
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<AudioMetadata> {
    let data = filesystem::read(path)?;

    Self::metadata(&data).context(error::Audio { path })
  }

  fn text_tag(ilst: Option<&Ilst>, fourcc: Fourcc, tag: &'static str) -> Result<Text, AudioError> {
    let values = match ilst {
      Some(ilst) => ilst
        .text(fourcc)
        .map_err(M4aError::from)
        .context(audio_error::M4aDecode)?,
      None => Vec::new(),
    };

    Audio::tag(values.into_iter(), tag)?
      .parse()
      .context(audio_error::TagInvalid { tag })
  }

  fn track(trak: &Trak) -> Result<AudioProperties, M4aError> {
    let timescale = trak.mdia.mdhd.timescale;

    ensure!(timescale != 0, m4a_error::TimescaleZero);

    let stbl = &trak.mdia.minf.stbl;

    let mp4a = match stbl.stsd.entries.first() {
      Some(SampleEntry::Mp4a(mp4a)) => mp4a,
      Some(SampleEntry::Avc1(_)) => {
        return Err(M4aError::CodecUnsupported {
          codec: "H.264".into(),
        });
      }
      Some(SampleEntry::Unknown(ty)) => {
        return Err(M4aError::CodecUnsupported {
          codec: ty.to_string(),
        });
      }
      None => {
        return Err(M4aError::CodecUnsupported {
          codec: "unknown".into(),
        });
      }
    };

    match mp4a.esds.object_type {
      0x40 | 0x66 | 0x67 => {}
      0x69 | 0x6b => {
        return Err(M4aError::CodecUnsupported {
          codec: "MP3".into(),
        });
      }
      _ => {
        return Err(M4aError::CodecUnsupported {
          codec: "unknown".into(),
        });
      }
    }

    let samples =
      u64::try_from(stbl.stts.duration() * u128::from(mp4a.sample_rate) / u128::from(timescale))
        .ok()
        .context(m4a_error::SamplesOverflow)?;

    ensure!(samples > 0, m4a_error::Empty);

    Ok(AudioProperties {
      channels: mp4a.channels.into(),
      sample_rate: mp4a.sample_rate.into(),
      samples,
      size: stbl.stsz.size(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn chain(error: &AudioError) -> String {
    error
      .iter_chain()
      .map(ToString::to_string)
      .collect::<Vec<String>>()
      .join(": ")
  }

  #[test]
  fn cover_art() {
    #[track_caller]
    fn case(builder: Mp4Builder, expected: &[(&[u8], Mime)]) {
      assert_eq!(
        M4aDecoder::cover_art(&builder.audio_track(0x40).build()).unwrap(),
        expected
          .iter()
          .map(|(data, media_type)| EmbeddedImage {
            data: data.to_vec(),
            media_type: media_type.clone(),
          })
          .collect::<Vec<EmbeddedImage>>(),
      );
    }

    case(Mp4Builder::new(), &[]);
    case(Mp4Builder::new().name("foo"), &[]);
    case(
      Mp4Builder::new().picture(13, b"foo"),
      &[(b"foo", mime::IMAGE_JPEG)],
    );
    case(
      Mp4Builder::new().picture(14, b"foo"),
      &[(b"foo", mime::IMAGE_PNG)],
    );
    case(
      Mp4Builder::new().picture(14, b"foo").picture(13, b"bar"),
      &[(b"foo", mime::IMAGE_PNG), (b"bar", mime::IMAGE_JPEG)],
    );

    assert_eq!(
      chain(&M4aDecoder::cover_art(&Mp4Builder::new().picture(0, b"foo").build()).unwrap_err()),
      "failed to decode MP4: unsupported cover art data type 0",
    );

    assert_eq!(
      chain(&M4aDecoder::cover_art(b"foo").unwrap_err()),
      "failed to decode MP4: truncated",
    );
  }

  fn m4a() -> Mp4Builder {
    Mp4Builder::new()
      .tag(*b"\xa9alb", "qux")
      .tag(*b"\xa9ART", "baz")
      .tag(*b"\xa9nam", "bar")
      .pair_tag(*b"disk", 1, 2)
      .pair_tag(*b"trkn", 3, 4)
      .frame_count(2)
  }

  #[test]
  fn metadata_err() {
    #[track_caller]
    fn case(builder: Mp4Builder, expected: &str) {
      assert_eq!(
        chain(&M4aDecoder::metadata(&builder.build()).unwrap_err()),
        expected,
      );
    }

    fn err(builder: Mp4Builder) -> AudioError {
      M4aDecoder::metadata(&builder.build()).unwrap_err()
    }

    assert_eq!(
      chain(&M4aDecoder::metadata(b"foo").unwrap_err()),
      "failed to decode MP4: truncated",
    );

    case(Mp4Builder::new(), "failed to decode MP4: no audio track");

    case(
      m4a().audio_track(0x40).audio_track(0x40),
      "failed to decode MP4: multiple audio tracks",
    );

    case(
      m4a().video_track(2, 1).audio_track(0x40),
      "failed to decode MP4: track 0 has unsupported track type `video`",
    );

    case(
      m4a().audio_track(0x40).track(*b"meta", 1000, &[]),
      "failed to decode MP4: track 1 has unsupported track type `metadata`",
    );

    case(
      m4a().audio_track(0x69),
      "failed to decode MP4: unsupported audio codec `MP3`",
    );

    case(
      m4a().audio_track(0x6b),
      "failed to decode MP4: unsupported audio codec `MP3`",
    );

    case(
      m4a().audio_track(0x11),
      "failed to decode MP4: unsupported audio codec `unknown`",
    );

    case(
      m4a().track(
        *b"soun",
        44100,
        &[Mp4Builder::video_entry(*b"fLaC", *b"dfLa", &[], 0, 0)],
      ),
      "failed to decode MP4: unsupported audio codec `fLaC`",
    );

    case(
      m4a().track(
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
      "failed to decode MP4: unsupported audio codec `H.264`",
    );

    case(
      m4a().track(*b"soun", 44100, &[]),
      "failed to decode MP4: unsupported audio codec `unknown`",
    );

    case(
      m4a().mp4a_version(2).audio_track(0x40),
      "failed to decode MP4: invalid `moov` box: invalid `trak` box: invalid `mdia` box: invalid `minf` box: invalid `stbl` box: invalid `stsd` box: invalid `mp4a` box: unsupported version 2",
    );

    case(
      m4a().media_timescale(0).audio_track(0x40),
      "failed to decode MP4: zero timescale",
    );

    case(
      m4a().frame_count(0).audio_track(0x40),
      "failed to decode MP4: no audio samples",
    );

    case(
      m4a().data(*b"\xa9alb", 0, b"qux").audio_track(0x40),
      "failed to decode MP4: `©alb` tag has unsupported data type 0",
    );

    case(
      Mp4Builder::new()
        .tag(*b"\xa9alb", "qux")
        .tag(*b"\xa9ART", "baz")
        .pair_tag(*b"disk", 1, 2)
        .tag(*b"\xa9nam", "bar")
        .data(*b"trkn", 1, b"qux")
        .frame_count(2)
        .audio_track(0x40),
      "failed to decode MP4: `trkn` tag has unsupported data type 1",
    );

    case(
      Mp4Builder::new()
        .tag(*b"\xa9alb", "qux")
        .tag(*b"\xa9ART", "baz")
        .data(*b"disk", 0, &[0; 4])
        .frame_count(2)
        .audio_track(0x40),
      "failed to decode MP4: invalid `disk` tag",
    );

    case(
      Mp4Builder::new()
        .tag(*b"\xa9alb", b"\xff")
        .frame_count(2)
        .audio_track(0x40),
      "failed to decode MP4: `©alb` tag is not valid UTF-8: invalid utf-8 sequence of 1 bytes from index 0",
    );

    assert_matches!(
      err(Mp4Builder::new().frame_count(2).audio_track(0x40)),
      AudioError::TagMissing { tag: "©alb" },
    );

    assert_matches!(
      err(m4a().tag(*b"\xa9alb", "quux").audio_track(0x40)),
      AudioError::TagMultiple { tag: "©alb" },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "")
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TagEmpty { tag: "©alb" },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "qux")
          .tag(*b"\xa9ART", "baz")
          .tag(*b"\xa9nam", "foo\tbar")
          .pair_tag(*b"disk", 1, 2)
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "©nam",
      },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "qux")
          .tag(*b"\xa9ART", "baz")
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TagMissing { tag: "disk" },
    );

    assert_matches!(
      err(m4a().pair_tag(*b"trkn", 3, 4).audio_track(0x40)),
      AudioError::TagMultiple { tag: "trkn" },
    );

    assert_matches!(
      err(
        Mp4Builder::new()
          .tag(*b"\xa9alb", "qux")
          .tag(*b"\xa9ART", "baz")
          .tag(*b"\xa9nam", "bar")
          .pair_tag(*b"disk", 1, 1)
          .pair_tag(*b"trkn", 1, 0)
          .frame_count(2)
          .audio_track(0x40)
      ),
      AudioError::TrackTotalMissing { tag: "trkn" },
    );
  }

  #[test]
  fn metadata_ok() {
    #[track_caller]
    fn case(builder: Mp4Builder, expected: AudioProperties) {
      let AudioMetadata {
        channels,
        sample_rate,
        samples,
        size,
        ..
      } = M4aDecoder::metadata(&builder.build()).unwrap();

      assert_eq!(
        AudioProperties {
          channels,
          sample_rate,
          samples,
          size,
        },
        expected,
      );
    }

    assert_eq!(
      M4aDecoder::metadata(&m4a().audio_track(0x40).build()).unwrap(),
      AudioMetadata {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        codec: AudioCodec::Aac,
        disc: 1,
        discs: 2,
        sample_bits: None,
        sample_rate: 44100,
        samples: 2,
        size: 2,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );

    let expected = AudioProperties {
      channels: 2,
      sample_rate: 44100,
      samples: 2,
      size: 2,
    };

    case(m4a().audio_track(0x66), expected.clone());
    case(m4a().audio_track(0x67), expected.clone());
    case(m4a().meta_in_moov().audio_track(0x40), expected);

    case(
      m4a().sample_sizes(&[3, 5]).audio_track(0x40),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 2,
        size: 8,
      },
    );

    case(
      m4a().media_timescale(22050).audio_track(0x40),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 4,
        size: 2,
      },
    );
  }

  #[test]
  fn read_ok() {
    let (_tempdir, root) = tempdir();

    let path = root.join("foo.m4a");

    std::fs::write(&path, m4a().audio_track(0x40).build()).unwrap();

    assert_eq!(
      M4aDecoder::read(&path).unwrap(),
      AudioMetadata {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        codec: AudioCodec::Aac,
        disc: 1,
        discs: 2,
        sample_bits: None,
        sample_rate: 44100,
        samples: 2,
        size: 2,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );
  }
}
