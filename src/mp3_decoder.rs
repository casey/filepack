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
  fn decode(data: &'a [u8]) -> Result<AudioProperties, Mp3Error> {
    let decoder = Self { data };

    let mut offset = 0;
    let mut first = Option::<Frame>::None;
    let mut samples = 0;

    while offset < decoder.data.len() {
      let frame = decoder.frame(offset)?;

      if !frame.metadata {
        samples += frame.samples;
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
    })
  }

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

  fn pair_tag(tag: &id3::Tag, path: &Utf8Path, id: &'static str) -> Result<(u64, u64)> {
    let value = Self::tag(tag, path, id)?;

    let (number, total) = value
      .split_once('/')
      .context(error::AudioTagPair { path, tag: id })?;

    Ok((
      parse_number(number).context(error::AudioTagInteger { path, tag: id })?,
      parse_number(total).context(error::AudioTagInteger { path, tag: id })?,
    ))
  }

  pub(crate) fn read(path: &Utf8Path) -> Result<AudioMetadata> {
    let data = filesystem::read(path)?;

    let tag = match id3::Tag::read_from2(io::Cursor::new(&data)) {
      Err(err) => {
        if let id3::ErrorKind::NoTag = err.kind {
          return Err(error::Mp3TagMissing { path }.build());
        }
        return Err(error::Mp3Tag { path }.into_error(err));
      }
      Ok(tag) => tag,
    };

    let album = Self::text_tag(&tag, path, "TALB")?;
    let artist = Self::text_tag(&tag, path, "TPE1")?;
    let (disc, discs) = Self::pair_tag(&tag, path, "TPOS")?;
    let title = Self::text_tag(&tag, path, "TIT2")?;
    let (track, tracks) = Self::pair_tag(&tag, path, "TRCK")?;

    let mut cursor = io::Cursor::new(&data);

    id3::Tag::skip(&mut cursor).context(error::Mp3Tag { path })?;

    let start = usize::try_from(cursor.position()).unwrap();

    let AudioProperties {
      channels,
      sample_rate,
      samples,
    } = Mp3Decoder::decode(&data[start..]).context(error::Mp3Decode { path })?;

    Ok(AudioMetadata {
      album,
      artist,
      channels,
      disc,
      discs,
      sample_bits: None,
      sample_rate,
      samples,
      title,
      track,
      tracks,
    })
  }

  fn tag<'t>(tag: &'t id3::Tag, path: &Utf8Path, id: &'static str) -> Result<&'t str> {
    audio_tag(
      tag
        .get(id)
        .and_then(|frame| frame.content().text_values())
        .into_iter()
        .flatten(),
      path,
      id,
    )
  }

  fn text_tag(tag: &id3::Tag, path: &Utf8Path, id: &'static str) -> Result<Text> {
    Self::tag(tag, path, id)?
      .parse()
      .context(error::AudioTagInvalid { path, tag: id })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode() {
    #[track_caller]
    fn case(data: &[Vec<u8>], expected: Result<AudioProperties, Mp3Error>) {
      assert_eq!(Mp3Decoder::decode(&data.concat()), expected);
    }

    fn frame(header: [u8; 4], size: usize) -> Vec<u8> {
      let mut bytes = header.to_vec();
      bytes.resize(size, 0);
      bytes
    }

    fn properties(channels: u64, sample_rate: u64, samples: u64) -> AudioProperties {
      AudioProperties {
        channels,
        sample_rate,
        samples,
      }
    }

    fn xing() -> Vec<u8> {
      let mut bytes = mp3_frame();
      bytes[36..40].copy_from_slice(b"Xing");
      bytes
    }

    let id3v1 = {
      let mut bytes = b"TAG".to_vec();
      bytes.resize(128, 0);
      bytes
    };

    case(&[mp3_frame(), mp3_frame()], Ok(properties(2, 44100, 2304)));

    case(
      &[xing(), mp3_frame(), mp3_frame()],
      Ok(properties(2, 44100, 2304)),
    );

    case(
      &[frame([0xFF, 0xFB, 0x92, 0x00], 418), mp3_frame()],
      Ok(properties(2, 44100, 2304)),
    );

    case(
      &[frame([0xFF, 0xFB, 0x90, 0xC0], 417)],
      Ok(properties(1, 44100, 1152)),
    );

    case(
      &[frame([0xFF, 0xF3, 0x90, 0x00], 261)],
      Ok(properties(2, 22050, 576)),
    );

    case(
      &[frame([0xFF, 0xE3, 0x90, 0x00], 522)],
      Ok(properties(2, 11025, 576)),
    );

    case(&[], Err(Mp3Error::Empty));

    case(&[xing()], Err(Mp3Error::Empty));

    case(&[mp3_frame(), id3v1], Err(Mp3Error::Sync { offset: 417 }));

    case(&[b"foobar".to_vec()], Err(Mp3Error::Sync { offset: 0 }));

    case(
      &[mp3_frame(), b"foobar".to_vec()],
      Err(Mp3Error::Sync { offset: 417 }),
    );

    case(&[mp3_frame()[..100].to_vec()], Err(Mp3Error::Truncated));

    case(&[mp3_frame(), b"bar".to_vec()], Err(Mp3Error::Truncated));

    case(
      &[frame([0xFF, 0xEB, 0x90, 0x00], 417)],
      Err(Mp3Error::Version),
    );

    case(
      &[frame([0xFF, 0xF9, 0x90, 0x00], 417)],
      Err(Mp3Error::LayerInvalid),
    );

    case(
      &[frame([0xFF, 0xFD, 0x90, 0x00], 417)],
      Err(Mp3Error::LayerUnsupported { layer: 2 }),
    );

    case(
      &[frame([0xFF, 0xFF, 0x90, 0x00], 417)],
      Err(Mp3Error::LayerUnsupported { layer: 1 }),
    );

    case(
      &[frame([0xFF, 0xFB, 0x00, 0x00], 417)],
      Err(Mp3Error::Bitrate { index: 0 }),
    );

    case(
      &[frame([0xFF, 0xFB, 0xF0, 0x00], 417)],
      Err(Mp3Error::Bitrate { index: 15 }),
    );

    case(
      &[frame([0xFF, 0xFB, 0x9C, 0x00], 417)],
      Err(Mp3Error::SampleRate),
    );

    case(
      &[mp3_frame(), frame([0xFF, 0xFB, 0x90, 0xC0], 417)],
      Err(Mp3Error::ChannelsMismatch {
        actual: 1,
        expected: 2,
      }),
    );

    case(
      &[mp3_frame(), frame([0xFF, 0xFB, 0x94, 0x00], 384)],
      Err(Mp3Error::SampleRateMismatch {
        actual: 48000,
        expected: 44100,
      }),
    );
  }

  #[test]
  fn read_err() {
    fn err(bytes: &[u8]) -> Error {
      let (_tempdir, root) = tempdir();

      let path = root.join("foo.mp3");

      std::fs::write(&path, bytes).unwrap();

      Mp3Decoder::read(&path).unwrap_err()
    }

    assert_matches!(err(b"foo"), Error::Mp3TagMissing { .. });

    assert_matches!(
      err(&mp3(&[], 1)),
      Error::AudioTagMissing { tag: "TALB", .. },
    );

    assert_matches!(
      err(&mp3(&["TALB=qux"], 1)),
      Error::AudioTagMissing { tag: "TPE1", .. },
    );

    assert_matches!(
      err(&mp3(&["TALB=qux\0quux"], 1)),
      Error::AudioTagMultiple { tag: "TALB", .. },
    );

    assert_matches!(
      err(&mp3(&["TALB="], 1)),
      Error::AudioTagEmpty { tag: "TALB", .. },
    );

    assert_matches!(
      err(&mp3(
        &["TALB=qux", "TIT2=foo\tbar", "TPE1=baz", "TPOS=1/2"],
        1
      )),
      Error::AudioTagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "TIT2",
        ..
      },
    );

    assert_matches!(
      err(&mp3(&["TALB=qux", "TPE1=baz", "TPOS=1"], 1)),
      Error::AudioTagPair { tag: "TPOS", .. },
    );

    assert_matches!(
      err(&mp3(
        &["TALB=qux", "TIT2=bar", "TPE1=baz", "TPOS=1/2", "TRCK=03/12"],
        1,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "TRCK",
        ..
      },
    );

    assert_matches!(
      err(&mp3(
        &["TALB=qux", "TIT2=bar", "TPE1=baz", "TPOS=1/2", "TRCK=3/4"],
        0,
      )),
      Error::Mp3Decode {
        source: Mp3Error::Empty,
        ..
      },
    );

    let mut bytes = mp3(
      &["TALB=qux", "TIT2=bar", "TPE1=baz", "TPOS=1/2", "TRCK=3/4"],
      0,
    );
    bytes.extend_from_slice(b"foobar");

    assert_matches!(
      err(&bytes),
      Error::Mp3Decode {
        source: Mp3Error::Sync { offset: 0 },
        ..
      },
    );
  }

  #[test]
  fn read_ok() {
    let (_tempdir, root) = tempdir();

    let path = root.join("foo.mp3");

    std::fs::write(
      &path,
      mp3(
        &["TALB=qux", "TIT2=bar", "TPE1=baz", "TPOS=1/2", "TRCK=3/4"],
        2,
      ),
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
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
      },
    );
  }
}
