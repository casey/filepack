use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Audio {
  #[n(0)]
  pub(crate) album: Text,
  #[n(1)]
  pub(crate) artist: Text,
  #[n(2)]
  pub(crate) channels: u64,
  #[n(3)]
  pub(crate) disc: u64,
  #[n(4)]
  pub(crate) discs: u64,
  #[n(5)]
  pub(crate) filename: ComponentBuf,
  #[n(6)]
  pub(crate) sample_bits: u64,
  #[n(7)]
  pub(crate) sample_rate: u64,
  #[n(8)]
  pub(crate) samples: u64,
  #[n(9)]
  pub(crate) title: Text,
  #[n(10)]
  pub(crate) track: u64,
  #[n(11)]
  pub(crate) tracks: u64,
  #[n(12)]
  #[serde(rename = "type")]
  pub(crate) ty: AudioType,
}

impl Audio {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  fn audio_info(reader: &FlacReader<fs::File>, path: &Utf8Path) -> Result<AudioInfo> {
    let streaminfo = reader.streaminfo();

    let samples = streaminfo
      .samples
      .context(error::AudioSampleCountUnknown { path })?;

    Ok(AudioInfo {
      channels: streaminfo.channels.into(),
      sample_bits: streaminfo.bits_per_sample.into(),
      sample_rate: streaminfo.sample_rate.into(),
      samples,
    })
  }

  pub(crate) fn check_content(&self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    match self.ty {
      AudioType::Flac => self.check_content_flac(&path),
    }
  }

  fn check_content_flac(&self, path: &Utf8Path) -> Result {
    let (_reader, audio_info) = Self::flac_reader(path)?;

    let AudioInfo {
      channels,
      sample_bits,
      sample_rate,
      samples,
    } = audio_info;

    ensure! {
      channels == self.channels,
      error::AudioChannelsMismatch {
        actual: channels,
        expected: self.channels,
        path,
      },
    }

    ensure! {
      sample_bits == self.sample_bits,
      error::AudioSampleBitsMismatch {
        actual: sample_bits,
        expected: self.sample_bits,
        path,
      },
    }

    ensure! {
      sample_rate == self.sample_rate,
      error::AudioSampleRateMismatch {
        actual: sample_rate,
        expected: self.sample_rate,
        path,
      },
    }

    ensure! {
      samples == self.samples,
      error::AudioSampleCountMismatch {
        actual: samples,
        expected: self.samples,
        path,
      },
    }

    Ok(())
  }

  pub(crate) fn check_positions(tracks: &[Audio]) -> Result<(), AudioError> {
    let Some(first) = tracks.first() else {
      return Ok(());
    };

    let discs = first.discs;

    let mut expected_disc = 1;
    let mut expected_track = 1;
    let mut disc_tracks = 0;

    for audio in tracks {
      ensure! {
        audio.discs == discs,
        audio_error::DiscTotalMismatch {
          actual: audio.discs,
          expected: discs,
          filename: audio.filename.clone(),
        },
      }

      ensure! {
        audio.disc == expected_disc && audio.track == expected_track,
        audio_error::PositionMismatch {
          disc: audio.disc,
          expected_disc,
          expected_track,
          filename: audio.filename.clone(),
          track: audio.track,
        },
      }

      ensure! {
        audio.disc <= discs,
        audio_error::DiscNumberExceedsTotal {
          filename: audio.filename.clone(),
          number: audio.disc,
          total: discs,
        },
      }

      if expected_track == 1 {
        disc_tracks = audio.tracks;
      } else {
        ensure! {
          audio.tracks == disc_tracks,
          audio_error::TotalMismatch {
            actual: audio.tracks,
            disc: expected_disc,
            expected: disc_tracks,
            filename: audio.filename.clone(),
          },
        }
      }

      ensure! {
        audio.track <= disc_tracks,
        audio_error::NumberExceedsTotal {
          filename: audio.filename.clone(),
          number: audio.track,
          total: disc_tracks,
        },
      }

      if expected_track == disc_tracks {
        expected_disc += 1;
        expected_track = 1;
      } else {
        expected_track += 1;
      }
    }

    ensure! {
      expected_disc == discs + 1,
      audio_error::Missing {
        disc: expected_disc,
        track: expected_track,
      },
    }

    Ok(())
  }

  pub(crate) fn duration(&self) -> Duration {
    if self.sample_rate == 0 {
      return Duration::ZERO;
    }

    let subsecond = u128::from(self.samples % self.sample_rate);

    Duration::new(
      self.samples / self.sample_rate,
      u32::try_from(subsecond * 1_000_000_000 / u128::from(self.sample_rate)).unwrap(),
    )
  }

  fn flac_reader(path: &Utf8Path) -> Result<(FlacReader<fs::File>, AudioInfo)> {
    let reader = FlacReader::open(path).context(error::AudioDecode { path })?;

    let audio_info = Self::audio_info(&reader, path)?;

    Ok((reader, audio_info))
  }

  pub(crate) fn format(&self) -> AudioFormat {
    AudioFormat {
      channels: self.channels,
      sample_bits: self.sample_bits,
      sample_rate: self.sample_rate,
      ty: self.ty,
    }
  }

  pub(crate) fn formats(tracks: &[Audio]) -> Vec<AudioFormat> {
    let mut formats = Vec::new();

    for audio in tracks {
      let format = audio.format();
      if !formats.contains(&format) {
        formats.push(format);
      }
    }

    formats
  }

  fn number_tag(reader: &FlacReader<fs::File>, path: &Utf8Path, tag: &'static str) -> Result<u64> {
    let value = Self::tag(reader, path, tag)?;
    parse_number(value).context(error::AudioTagInteger { path, tag })
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    match self.ty {
      AudioType::Flac => self.populate_flac(&path),
    }
  }

  fn populate_flac(&mut self, path: &Utf8Path) -> Result {
    let (reader, audio_info) = Self::flac_reader(path)?;

    let AudioInfo {
      channels,
      sample_bits,
      sample_rate,
      samples,
    } = audio_info;

    self.channels = channels;
    self.sample_bits = sample_bits;
    self.sample_rate = sample_rate;
    self.samples = samples;

    self.album = Self::text_tag(&reader, path, "album")?;
    self.artist = Self::text_tag(&reader, path, "artist")?;
    self.disc = Self::number_tag(&reader, path, "discnumber")?;
    self.discs = Self::number_tag(&reader, path, "disctotal")?;
    self.title = Self::text_tag(&reader, path, "title")?;
    self.track = Self::number_tag(&reader, path, "tracknumber")?;
    self.tracks = Self::number_tag(&reader, path, "tracktotal")?;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }

  pub(crate) fn sum_durations(tracks: &[Audio]) -> Duration {
    tracks.iter().fold(Duration::ZERO, |sum, audio| {
      sum.saturating_add(audio.duration())
    })
  }

  fn tag<'a>(
    reader: &'a FlacReader<fs::File>,
    path: &Utf8Path,
    tag: &'static str,
  ) -> Result<&'a str> {
    let mut values = reader.get_tag(tag);

    let value = values
      .next()
      .context(error::AudioTagMissing { path, tag })?;

    ensure! {
      values.next().is_none(),
      error::AudioTagMultiple { path, tag },
    }

    ensure! {
      !value.is_empty(),
      error::AudioTagEmpty { path, tag },
    }

    Ok(value)
  }

  fn text_tag(reader: &FlacReader<fs::File>, path: &Utf8Path, tag: &'static str) -> Result<Text> {
    Self::tag(reader, path, tag)?
      .parse()
      .context(error::AudioTagInvalid { path, tag })
  }
}

impl FromStr for Audio {
  type Err = ComponentError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let filename = s.parse::<ComponentBuf>()?;

    let Some(ty) = filename.extension().and_then(AudioType::from_extension) else {
      return Err(ComponentError::Extension {
        extensions: AudioType::EXTENSIONS,
      });
    };

    Ok(Self {
      album: Text::new(),
      artist: Text::new(),
      channels: 0,
      disc: 0,
      discs: 0,
      filename,
      sample_bits: 0,
      sample_rate: 0,
      samples: 0,
      title: Text::new(),
      track: 0,
      tracks: 0,
      ty,
    })
  }
}

impl Item for Audio {
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
  fn check_content() {
    #[track_caller]
    fn case(audio: &Audio) -> Result {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.flac"), flac(&[], 44100)).unwrap();

      audio.check_content(&root)
    }

    let mut audio = "foo.flac".parse::<Audio>().unwrap();
    audio.channels = 2;
    audio.sample_bits = 16;
    audio.sample_rate = 44100;
    audio.samples = 44100;

    case(&audio).unwrap();

    audio.samples = 1;

    assert_matches_regex!(
      case(&audio).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has 44100 samples but metadata sample count is 1$",
    );

    audio.samples = 44100;
    audio.sample_rate = 22050;

    assert_matches_regex!(
      case(&audio).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has sample rate 44100 but metadata sample rate is 22050$",
    );

    audio.sample_rate = 44100;
    audio.sample_bits = 24;

    assert_matches_regex!(
      case(&audio).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has 16 bits per sample but metadata sample bits is 24$",
    );

    audio.sample_bits = 16;
    audio.channels = 1;

    assert_matches_regex!(
      case(&audio).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has 2 channels but metadata channel count is 1$",
    );
  }

  #[test]
  fn check_positions() {
    #[track_caller]
    fn case(positions: &[(u64, u64, u64, u64)], expected: Result<(), AudioError>) {
      let tracks = positions
        .iter()
        .enumerate()
        .map(|(i, (disc, discs, track, tracks))| {
          let mut t = format!("{i}.flac").parse::<Audio>().unwrap();
          t.disc = *disc;
          t.discs = *discs;
          t.track = *track;
          t.tracks = *tracks;
          t
        })
        .collect::<Vec<Audio>>();

      assert_eq!(Audio::check_positions(&tracks), expected);
    }

    case(&[], Ok(()));

    case(&[(1, 1, 1, 1)], Ok(()));

    case(&[(1, 2, 1, 2), (1, 2, 2, 2), (2, 2, 1, 1)], Ok(()));

    case(
      &[(1, 1, 2, 2), (1, 1, 1, 2)],
      Err(AudioError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        filename: "0.flac".parse().unwrap(),
        track: 2,
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 1, 2)],
      Err(AudioError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        filename: "1.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 1, 3), (1, 1, 3, 3)],
      Err(AudioError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        filename: "1.flac".parse().unwrap(),
        track: 3,
      }),
    );

    case(
      &[(1, 1, 1, 2)],
      Err(AudioError::Missing { disc: 1, track: 2 }),
    );

    case(
      &[(1, 2, 1, 1)],
      Err(AudioError::Missing { disc: 2, track: 1 }),
    );

    case(
      &[(1, 2, 1, 1), (2, 1, 1, 1)],
      Err(AudioError::DiscTotalMismatch {
        actual: 1,
        expected: 2,
        filename: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 2, 3)],
      Err(AudioError::TotalMismatch {
        actual: 3,
        disc: 1,
        expected: 2,
        filename: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 1), (2, 1, 1, 1)],
      Err(AudioError::DiscNumberExceedsTotal {
        filename: "1.flac".parse().unwrap(),
        number: 2,
        total: 1,
      }),
    );

    case(
      &[(1, 0, 1, 1)],
      Err(AudioError::DiscNumberExceedsTotal {
        filename: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(1, 1, 1, 0)],
      Err(AudioError::NumberExceedsTotal {
        filename: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(0, 1, 1, 1)],
      Err(AudioError::PositionMismatch {
        disc: 0,
        expected_disc: 1,
        expected_track: 1,
        filename: "0.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 0, 1)],
      Err(AudioError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        filename: "0.flac".parse().unwrap(),
        track: 0,
      }),
    );
  }

  #[test]
  fn duration() {
    #[track_caller]
    fn case(samples: u64, sample_rate: u64, expected: Duration) {
      let mut audio = "foo.flac".parse::<Audio>().unwrap();
      audio.sample_rate = sample_rate;
      audio.samples = samples;
      assert_eq!(audio.duration(), expected);
    }

    case(0, 0, Duration::ZERO);
    case(44100, 44100, Duration::from_secs(1));
    case(66150, 44100, Duration::from_millis(1500));
    case(u64::MAX, u64::MAX - 1, Duration::new(1, 0));
  }

  #[test]
  fn encoding() {
    assert_cbor(
      "foo.flac".parse::<Audio>().unwrap(),
      "ad006001600200030004000568666f6f2e666c616306000700080009600a000b000c00",
    );

    assert_cbor(
      Audio {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 8,
        disc: 3,
        discs: 4,
        filename: "foo.flac".parse().unwrap(),
        sample_bits: 7,
        sample_rate: 1,
        samples: 2,
        title: "bar".parse().unwrap(),
        track: 5,
        tracks: 6,
        ty: AudioType::Flac,
      },
      "ad0063717578016362617a0208030304040568666f6f2e666c616306070701080209636261720a050b060c00",
    );
  }

  #[test]
  fn format() {
    let mut audio = "foo.flac".parse::<Audio>().unwrap();
    audio.channels = 2;
    audio.sample_bits = 16;
    audio.sample_rate = 44100;

    assert_eq!(
      audio.format(),
      AudioFormat {
        channels: 2,
        sample_bits: 16,
        sample_rate: 44100,
        ty: AudioType::Flac,
      },
    );
  }

  #[test]
  fn formats() {
    let mut foo = "foo.flac".parse::<Audio>().unwrap();
    foo.channels = 2;
    foo.sample_bits = 16;
    foo.sample_rate = 44100;

    let mut bar = foo.clone();
    bar.sample_bits = 24;

    assert_eq!(
      Audio::formats(&[foo.clone(), bar.clone(), foo.clone()]),
      [foo.format(), bar.format()],
    );
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: ComponentError) {
      assert_eq!(s.parse::<Audio>().unwrap_err(), expected);
    }

    assert_eq!(
      "foo.flac".parse::<Audio>().unwrap(),
      Audio {
        album: Text::new(),
        artist: Text::new(),
        channels: 0,
        disc: 0,
        discs: 0,
        filename: "foo.flac".parse().unwrap(),
        sample_bits: 0,
        sample_rate: 0,
        samples: 0,
        title: Text::new(),
        track: 0,
        tracks: 0,
        ty: AudioType::Flac,
      },
    );

    case(
      "foo.mp3",
      ComponentError::Extension {
        extensions: &["flac"],
      },
    );
    case(
      "foo",
      ComponentError::Extension {
        extensions: &["flac"],
      },
    );
    case("", ComponentError::Empty);
    case("foo/bar.flac", ComponentError::Separator { character: '/' });
  }

  #[test]
  fn populate_err() {
    fn err(bytes: &[u8]) -> Error {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.flac"), bytes).unwrap();

      let mut audio = "foo.flac".parse::<Audio>().unwrap();

      audio.populate(&root).unwrap_err()
    }

    assert_matches!(err(b"foo"), Error::AudioDecode { .. });

    assert_matches!(
      err(&flac(&[], 44100)),
      Error::AudioTagMissing { tag: "album", .. },
    );

    assert_matches!(
      err(&flac(&["ALBUM=qux", "TITLE=bar"], 44100)),
      Error::AudioTagMissing { tag: "artist", .. },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ARTIST=baz", "DISCNUMBER=1", "DISCTOTAL=1"],
        44100,
      )),
      Error::AudioTagMissing { tag: "title", .. },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ALBUM=quux", "ARTIST=baz", "TITLE=bar"],
        44100,
      )),
      Error::AudioTagMultiple { tag: "album", .. },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE="
        ],
        44100,
      )),
      Error::AudioTagEmpty { tag: "title", .. },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=foo\tbar",
        ],
        44100,
      )),
      Error::AudioTagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "title",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
        ],
        44100,
      )),
      Error::AudioTagMissing {
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=foo",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=3/12",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=01",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=+1",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Invalid { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=18446744073709551616",
        ],
        44100,
      )),
      Error::AudioTagInteger {
        source: NumberError::Integer { .. },
        tag: "tracknumber",
        ..
      },
    );

    assert_matches!(
      err(&flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=1",
          "TRACKTOTAL=1",
        ],
        0,
      )),
      Error::AudioSampleCountUnknown { .. },
    );
  }

  #[test]
  fn populate_ok() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.flac"),
      flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=2",
          "TITLE=bar",
          "TRACKNUMBER=3",
          "TRACKTOTAL=4",
        ],
        66150,
      ),
    )
    .unwrap();

    let mut audio = "foo.flac".parse::<Audio>().unwrap();
    audio.populate(&root).unwrap();

    assert_eq!(audio.album.as_str(), "qux");
    assert_eq!(audio.artist.as_str(), "baz");
    assert_eq!(audio.channels, 2);
    assert_eq!(audio.disc, 1);
    assert_eq!(audio.discs, 2);
    assert_eq!(audio.sample_bits, 16);
    assert_eq!(audio.sample_rate, 44100);
    assert_eq!(audio.samples, 66150);
    assert_eq!(audio.title.as_str(), "bar");
    assert_eq!(audio.track, 3);
    assert_eq!(audio.tracks, 4);
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Audio>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","channels":0,"disc":0,"discs":0,"filename":"foo.flac","sample_bits":0,"sample_rate":0,"samples":0,"title":"","track":0,"tracks":0,"type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Audio {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 8,
        disc: 3,
        discs: 4,
        filename: "foo.flac".parse().unwrap(),
        sample_bits: 7,
        sample_rate: 1,
        samples: 2,
        title: "bar".parse().unwrap(),
        track: 5,
        tracks: 6,
        ty: AudioType::Flac,
      })
      .unwrap(),
      r#"{"album":"qux","artist":"baz","channels":8,"disc":3,"discs":4,"filename":"foo.flac","sample_bits":7,"sample_rate":1,"samples":2,"title":"bar","track":5,"tracks":6,"type":"flac"}"#,
    );
  }
}
