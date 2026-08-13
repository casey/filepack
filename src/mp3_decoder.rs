use super::*;

const MPEG1_BITRATES: [u64; 14] = [
  32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
];

const MPEG2_BITRATES: [u64; 14] = [8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];

const SAMPLE_RATES: [u64; 3] = [44100, 48000, 32000];

#[derive(Debug, PartialEq)]
struct AudioProperties {
  channels: u64,
  sample_rate: u64,
  samples: u64,
  size: u64,
}

struct Frame {
  channels: u64,
  metadata: bool,
  sample_rate: u64,
  samples: u64,
  size: usize,
}

pub(crate) struct Mp3Decoder<'a> {
  data: &'a [u8],
}

#[derive(Clone, Copy)]
enum Version {
  Mpeg1,
  Mpeg2,
  Mpeg25,
}

impl Version {
  fn bitrates(self) -> [u64; 14] {
    match self {
      Self::Mpeg1 => MPEG1_BITRATES,
      Self::Mpeg2 | Self::Mpeg25 => MPEG2_BITRATES,
    }
  }

  fn divisor(self) -> u64 {
    match self {
      Self::Mpeg1 => 1,
      Self::Mpeg2 => 2,
      Self::Mpeg25 => 4,
    }
  }

  fn samples(self) -> u64 {
    match self {
      Self::Mpeg1 => 1152,
      Self::Mpeg2 | Self::Mpeg25 => 576,
    }
  }
}

impl<'a> Mp3Decoder<'a> {
  fn frame(&self, offset: usize) -> Result<Frame, Mp3Error> {
    let header = self
      .data
      .get(offset..offset + 4)
      .context(mp3_error::Truncated)?;

    ensure! {
      header[0] == 0xFF && header[1] & 0xE0 == 0xE0,
      mp3_error::Sync { offset },
    }

    let version = match (header[1] >> 3) & 0b11 {
      0 => Version::Mpeg25,
      2 => Version::Mpeg2,
      3 => Version::Mpeg1,
      _ => return Err(mp3_error::Version.build()),
    };

    match (header[1] >> 1) & 0b11 {
      0 => return Err(mp3_error::LayerInvalid.build()),
      1 => {}
      bits => return Err(mp3_error::LayerUnsupported { layer: 4 - bits }.build()),
    }

    let index = header[2] >> 4;

    ensure!((1..=14).contains(&index), mp3_error::Bitrate { index });

    let bitrate = version.bitrates()[usize::from(index) - 1] * 1000;

    let sample_rate = match (header[2] >> 2) & 0b11 {
      3 => return Err(mp3_error::SampleRate.build()),
      index => SAMPLE_RATES[usize::from(index)] / version.divisor(),
    };

    let padding = u64::from((header[2] >> 1) & 1);

    let channels = if header[3] >> 6 == 3 { 1 } else { 2 };

    let samples = version.samples();

    let size = usize::try_from(samples / 8 * bitrate / sample_rate + padding).unwrap();

    ensure!(self.data.len() - offset >= size, mp3_error::Truncated);

    let side_info = match (version, channels == 1) {
      (Version::Mpeg1, false) => 32,
      (Version::Mpeg1, true) | (Version::Mpeg2 | Version::Mpeg25, false) => 17,
      (Version::Mpeg2 | Version::Mpeg25, true) => 9,
    };

    let metadata = matches!(
      self
        .data
        .get(offset + 4 + side_info..offset + 8 + side_info),
      Some(b"Xing" | b"Info"),
    );

    Ok(Frame {
      channels,
      metadata,
      sample_rate,
      samples,
      size,
    })
  }

  fn metadata(data: &[u8]) -> Result<AudioMetadata, AudioError> {
    let tag = match id3::Tag::read_from2(io::Cursor::new(data)) {
      Err(err) => {
        if let id3::ErrorKind::NoTag = err.kind {
          return Err(audio_error::Mp3TagMissing.build());
        }
        return Err(audio_error::Mp3Tag.into_error(err));
      }
      Ok(tag) => tag,
    };

    let album = Self::text_tag(&tag, "TALB")?;
    let artist = Self::text_tag(&tag, "TPE1")?;
    let (disc, discs) = Self::pair_tag(&tag, "TPOS")?;
    let title = Self::text_tag(&tag, "TIT2")?;
    let (track, tracks) = Self::pair_tag(&tag, "TRCK")?;

    let mut cursor = io::Cursor::new(data);

    id3::Tag::skip(&mut cursor).context(audio_error::Mp3Tag)?;

    let start = usize::try_from(cursor.position()).unwrap();

    let AudioProperties {
      channels,
      sample_rate,
      samples,
      size,
    } = Mp3Decoder::properties(&data[start..]).context(audio_error::Mp3Decode)?;

    Ok(AudioMetadata {
      album,
      artist,
      channels,
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

  fn pair_tag(tag: &id3::Tag, id: &'static str) -> Result<(u64, u64), AudioError> {
    let value = Self::tag(tag, id)?;

    let (number, total) = value
      .split_once('/')
      .context(audio_error::TagPair { tag: id })?;

    Ok((
      parse_number(number).context(audio_error::TagInteger { tag: id })?,
      parse_number(total).context(audio_error::TagInteger { tag: id })?,
    ))
  }

  fn properties(data: &'a [u8]) -> Result<AudioProperties, Mp3Error> {
    let decoder = Self { data };

    let mut offset = 0;
    let mut first = Option::<Frame>::None;
    let mut samples = 0;
    let mut size = 0;

    while offset < decoder.data.len() {
      let frame = decoder.frame(offset)?;

      if !frame.metadata {
        samples += frame.samples;
        size += frame.size.into_u64();
      }

      offset += frame.size;

      if let Some(first) = &first {
        ensure! {
          frame.channels == first.channels,
          mp3_error::ChannelsMismatch {
            actual: frame.channels,
            expected: first.channels,
          },
        }

        ensure! {
          frame.sample_rate == first.sample_rate,
          mp3_error::SampleRateMismatch {
            actual: frame.sample_rate,
            expected: first.sample_rate,
          },
        }
      } else {
        first = Some(frame);
      }
    }

    let Frame {
      channels,
      sample_rate,
      ..
    } = first.context(mp3_error::Empty)?;

    ensure!(samples > 0, mp3_error::Empty);

    Ok(AudioProperties {
      channels,
      sample_rate,
      samples,
      size,
    })
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<AudioMetadata> {
    let data = filesystem::read(path)?;

    Self::metadata(&data).context(error::Audio { path })
  }

  fn tag<'t>(tag: &'t id3::Tag, id: &'static str) -> Result<&'t str, AudioError> {
    Audio::tag(
      tag
        .get(id)
        .and_then(|frame| frame.content().text_values())
        .into_iter()
        .flatten(),
      id,
    )
  }

  fn text_tag(tag: &id3::Tag, id: &'static str) -> Result<Text, AudioError> {
    Self::tag(tag, id)?
      .parse()
      .context(audio_error::TagInvalid { tag: id })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn metadata_err() {
    fn err(builder: Mp3Builder) -> AudioError {
      Mp3Decoder::metadata(&builder.build()).unwrap_err()
    }

    assert_matches!(
      err(Mp3Builder::new().trailing(b"foo")),
      AudioError::Mp3TagMissing,
    );

    assert_matches!(
      err(Mp3Builder::new().id3v2().frames(1)),
      AudioError::TagMissing { tag: "TALB" },
    );

    assert_matches!(
      err(Mp3Builder::new().tag("TALB", "qux").frames(1)),
      AudioError::TagMissing { tag: "TPE1" },
    );

    assert_matches!(
      err(Mp3Builder::new().tag("TALB", "qux\0quux").frames(1)),
      AudioError::TagMultiple { tag: "TALB" },
    );

    assert_matches!(
      err(Mp3Builder::new().tag("TALB", "").frames(1)),
      AudioError::TagEmpty { tag: "TALB" },
    );

    assert_matches!(
      err(
        Mp3Builder::new()
          .tag("TALB", "qux")
          .tag("TIT2", "foo\tbar")
          .tag("TPE1", "baz")
          .tag("TPOS", "1/2")
          .frames(1)
      ),
      AudioError::TagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "TIT2",
      },
    );

    assert_matches!(
      err(
        Mp3Builder::new()
          .tag("TALB", "qux")
          .tag("TPE1", "baz")
          .tag("TPOS", "1")
          .frames(1)
      ),
      AudioError::TagPair { tag: "TPOS" },
    );

    assert_matches!(
      err(
        Mp3Builder::new()
          .tag("TALB", "qux")
          .tag("TIT2", "bar")
          .tag("TPE1", "baz")
          .tag("TPOS", "1/2")
          .tag("TRCK", "03/12")
          .frames(1)
      ),
      AudioError::TagInteger {
        source: NumberError::Invalid { .. },
        tag: "TRCK",
      },
    );
  }

  #[test]
  fn properties_err() {
    #[track_caller]
    fn case(builder: Mp3Builder, expected: Mp3Error) {
      let mp3 = builder
        .tag("TALB", "qux")
        .tag("TIT2", "bar")
        .tag("TPE1", "baz")
        .tag("TPOS", "1/2")
        .tag("TRCK", "3/4")
        .build();

      match Mp3Decoder::metadata(&mp3).unwrap_err() {
        AudioError::Mp3Decode { source } => assert_eq!(source, expected),
        err => panic!("unexpected error: {err}"),
      }
    }

    case(Mp3Builder::new(), Mp3Error::Empty);

    case(Mp3Builder::new().xing(), Mp3Error::Empty);

    case(
      Mp3Builder::new().frames(1).id3v1(),
      Mp3Error::Sync { offset: 417 },
    );

    case(
      Mp3Builder::new().trailing(b"foobar"),
      Mp3Error::Sync { offset: 0 },
    );

    case(
      Mp3Builder::new().frames(1).trailing(b"foobar"),
      Mp3Error::Sync { offset: 417 },
    );

    case(
      Mp3Builder::new().frames(1).truncate(180),
      Mp3Error::Truncated,
    );

    case(
      Mp3Builder::new().frames(1).trailing(b"bar"),
      Mp3Error::Truncated,
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xEB, 0x90, 0x00], 417),
      Mp3Error::Version,
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xF9, 0x90, 0x00], 417),
      Mp3Error::LayerInvalid,
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xFD, 0x90, 0x00], 417),
      Mp3Error::LayerUnsupported { layer: 2 },
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xFF, 0x90, 0x00], 417),
      Mp3Error::LayerUnsupported { layer: 1 },
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xFB, 0x00, 0x00], 417),
      Mp3Error::Bitrate { index: 0 },
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xFB, 0xF0, 0x00], 417),
      Mp3Error::Bitrate { index: 15 },
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xFB, 0x9C, 0x00], 417),
      Mp3Error::SampleRate,
    );

    case(
      Mp3Builder::new()
        .frames(1)
        .frame([0xFF, 0xFB, 0x90, 0xC0], 417),
      Mp3Error::ChannelsMismatch {
        actual: 1,
        expected: 2,
      },
    );

    case(
      Mp3Builder::new()
        .frames(1)
        .frame([0xFF, 0xFB, 0x94, 0x00], 384),
      Mp3Error::SampleRateMismatch {
        actual: 48000,
        expected: 44100,
      },
    );
  }

  #[test]
  fn properties_ok() {
    #[track_caller]
    fn case(builder: Mp3Builder, expected: AudioProperties) {
      let mp3 = builder
        .tag("TALB", "qux")
        .tag("TIT2", "bar")
        .tag("TPE1", "baz")
        .tag("TPOS", "1/2")
        .tag("TRCK", "3/4")
        .build();

      let AudioMetadata {
        channels,
        sample_rate,
        samples,
        size,
        ..
      } = Mp3Decoder::metadata(&mp3).unwrap();

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

    case(
      Mp3Builder::new().frames(2),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 2304,
        size: 834,
      },
    );

    case(
      Mp3Builder::new().xing().frames(2),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 2304,
        size: 834,
      },
    );

    case(
      Mp3Builder::new()
        .frame([0xFF, 0xFB, 0x92, 0x00], 418)
        .frames(1),
      AudioProperties {
        channels: 2,
        sample_rate: 44100,
        samples: 2304,
        size: 835,
      },
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xFB, 0x90, 0xC0], 417),
      AudioProperties {
        channels: 1,
        sample_rate: 44100,
        samples: 1152,
        size: 417,
      },
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xF3, 0x90, 0x00], 261),
      AudioProperties {
        channels: 2,
        sample_rate: 22050,
        samples: 576,
        size: 261,
      },
    );

    case(
      Mp3Builder::new().frame([0xFF, 0xE3, 0x90, 0x00], 522),
      AudioProperties {
        channels: 2,
        sample_rate: 11025,
        samples: 576,
        size: 522,
      },
    );
  }

  #[test]
  fn read_ok() {
    let (_tempdir, root) = tempdir();

    let path = root.join("foo.mp3");

    std::fs::write(
      &path,
      Mp3Builder::new()
        .tag("TALB", "qux")
        .tag("TIT2", "bar")
        .tag("TPE1", "baz")
        .tag("TPOS", "1/2")
        .tag("TRCK", "3/4")
        .frames(2)
        .build(),
    )
    .unwrap();

    assert_eq!(
      Mp3Decoder::read(&path).unwrap(),
      AudioMetadata {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        disc: 1,
        discs: 2,
        sample_bits: None,
        sample_rate: 44100,
        samples: 2304,
        size: 834,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );
  }
}
