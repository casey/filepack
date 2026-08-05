use super::*;

#[skip_serializing_none]
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
  pub(crate) sample_bits: Option<u64>,
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

  fn flac_info(reader: &FlacReader<fs::File>, path: &Utf8Path) -> Result<AudioInfo> {
    let streaminfo = reader.streaminfo();

    let samples = streaminfo
      .samples
      .context(error::FlacSampleCountUnknown { path })?;

    Ok(AudioInfo {
      channels: streaminfo.channels.into(),
      sample_bits: Some(streaminfo.bits_per_sample.into()),
      sample_rate: streaminfo.sample_rate.into(),
      samples,
    })
  }

  fn flac_number_tag(
    reader: &FlacReader<fs::File>,
    path: &Utf8Path,
    tag: &'static str,
  ) -> Result<u64> {
    let value = Self::flac_tag(reader, path, tag)?;
    parse_number(value).context(error::AudioTagInteger { path, tag })
  }

  fn flac_reader(path: &Utf8Path) -> Result<(FlacReader<fs::File>, AudioInfo)> {
    let reader = FlacReader::open(path).context(error::FlacDecode { path })?;

    let audio_info = Self::flac_info(&reader, path)?;

    Ok((reader, audio_info))
  }

  fn flac_tag<'a>(
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

  fn flac_text_tag(
    reader: &FlacReader<fs::File>,
    path: &Utf8Path,
    tag: &'static str,
  ) -> Result<Text> {
    Self::flac_tag(reader, path, tag)?
      .parse()
      .context(error::AudioTagInvalid { path, tag })
  }

  pub(crate) fn formats(tracks: &[Audio]) -> Vec<AudioType> {
    let mut formats = Vec::new();

    for audio in tracks {
      if !formats.contains(&audio.ty) {
        formats.push(audio.ty);
      }
    }

    formats
  }

  fn id3_pair_tag(tag: &id3::Tag, path: &Utf8Path, id: &'static str) -> Result<(u64, u64)> {
    let value = Self::id3_tag(tag, path, id)?;

    let (number, total) = value
      .split_once('/')
      .context(error::AudioTagPair { path, tag: id })?;

    Ok((
      parse_number(number).context(error::AudioTagInteger { path, tag: id })?,
      parse_number(total).context(error::AudioTagInteger { path, tag: id })?,
    ))
  }

  fn id3_tag<'a>(tag: &'a id3::Tag, path: &Utf8Path, id: &'static str) -> Result<&'a str> {
    let mut values = tag
      .get(id)
      .and_then(|frame| frame.content().text_values())
      .into_iter()
      .flatten();

    let value = values
      .next()
      .context(error::AudioTagMissing { path, tag: id })?;

    ensure! {
      values.next().is_none(),
      error::AudioTagMultiple { path, tag: id },
    }

    ensure! {
      !value.is_empty(),
      error::AudioTagEmpty { path, tag: id },
    }

    Ok(value)
  }

  fn id3_text_tag(tag: &id3::Tag, path: &Utf8Path, id: &'static str) -> Result<Text> {
    Self::id3_tag(tag, path, id)?
      .parse()
      .context(error::AudioTagInvalid { path, tag: id })
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    match self.ty {
      AudioType::Flac => self.populate_flac(&path),
      AudioType::Mp3 => self.populate_mp3(&path),
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

    self.album = Self::flac_text_tag(&reader, path, "album")?;
    self.artist = Self::flac_text_tag(&reader, path, "artist")?;
    self.disc = Self::flac_number_tag(&reader, path, "discnumber")?;
    self.discs = Self::flac_number_tag(&reader, path, "disctotal")?;
    self.title = Self::flac_text_tag(&reader, path, "title")?;
    self.track = Self::flac_number_tag(&reader, path, "tracknumber")?;
    self.tracks = Self::flac_number_tag(&reader, path, "tracktotal")?;

    Ok(())
  }

  fn populate_mp3(&mut self, path: &Utf8Path) -> Result {
    let data = filesystem::read(path)?;

    let tag = match id3::Tag::read_from2(io::Cursor::new(&data)) {
      Err(err) if matches!(err.kind, id3::ErrorKind::NoTag) => {
        return Err(error::Mp3TagMissing { path }.build());
      }
      result => result.context(error::Mp3Tag { path })?,
    };

    self.album = Self::id3_text_tag(&tag, path, "TALB")?;
    self.artist = Self::id3_text_tag(&tag, path, "TPE1")?;
    (self.disc, self.discs) = Self::id3_pair_tag(&tag, path, "TPOS")?;
    self.title = Self::id3_text_tag(&tag, path, "TIT2")?;
    (self.track, self.tracks) = Self::id3_pair_tag(&tag, path, "TRCK")?;

    let mut cursor = io::Cursor::new(&data);

    id3::Tag::skip(&mut cursor).context(error::Mp3Tag { path })?;

    let start = usize::try_from(cursor.position()).unwrap();

    let AudioInfo {
      channels,
      sample_bits,
      sample_rate,
      samples,
    } = Mp3Decoder::decode(&data[start..]).context(error::Mp3Decode { path })?;

    self.channels = channels;
    self.sample_bits = sample_bits;
    self.sample_rate = sample_rate;
    self.samples = samples;

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
      sample_bits: None,
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
  fn info(&self, url: String) -> Info {
    let mut map = vec![
      (
        "filename".into(),
        Info::Link {
          text: self.filename.to_string(),
          url,
        },
      ),
      ("title".into(), Info::Value(self.title.to_string())),
      ("artist".into(), Info::Value(self.artist.to_string())),
      ("album".into(), Info::Value(self.album.to_string())),
      (
        "disc".into(),
        Info::Value(format!("{} of {}", self.disc, self.discs)),
      ),
      (
        "track".into(),
        Info::Value(format!("{} of {}", self.track, self.tracks)),
      ),
      (
        "duration".into(),
        Info::Value(DisplayDuration(self.duration()).to_string()),
      ),
      ("type".into(), Info::Value(self.ty.to_string())),
    ];

    if let Some(sample_bits) = self.sample_bits {
      map.push((
        "sample bits".into(),
        Info::Value(format!("{sample_bits}-bit")),
      ));
    }

    map.extend([
      (
        "sample rate".into(),
        Info::Value(DisplaySampleRate(self.sample_rate).to_string()),
      ),
      ("channels".into(), Info::Value(self.channels.to_string())),
      (
        "compression mode".into(),
        Info::Value(
          match self.ty {
            AudioType::Flac => "lossless",
            AudioType::Mp3 => "lossy",
          }
          .into(),
        ),
      ),
      ("samples".into(), Info::Value(self.samples.to_string())),
    ]);

    Info::Map(map)
  }

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
  fn formats() {
    let foo = "foo.flac".parse::<Audio>().unwrap();
    let bar = "bar.flac".parse::<Audio>().unwrap();
    let baz = "baz.mp3".parse::<Audio>().unwrap();

    assert_eq!(
      Audio::formats(&[foo, bar, baz]),
      [AudioType::Flac, AudioType::Mp3],
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
        sample_bits: None,
        sample_rate: 0,
        samples: 0,
        title: Text::new(),
        track: 0,
        tracks: 0,
        ty: AudioType::Flac,
      },
    );

    assert_eq!(
      "foo.mp3".parse::<Audio>().unwrap(),
      Audio {
        album: Text::new(),
        artist: Text::new(),
        channels: 0,
        disc: 0,
        discs: 0,
        filename: "foo.mp3".parse().unwrap(),
        sample_bits: None,
        sample_rate: 0,
        samples: 0,
        title: Text::new(),
        track: 0,
        tracks: 0,
        ty: AudioType::Mp3,
      },
    );

    case(
      "foo.wav",
      ComponentError::Extension {
        extensions: &["flac", "mp3"],
      },
    );
    case(
      "foo",
      ComponentError::Extension {
        extensions: &["flac", "mp3"],
      },
    );
    case("", ComponentError::Empty);
    case("foo/bar.flac", ComponentError::Separator { character: '/' });
  }

  #[test]
  fn info() {
    let mut audio = "foo.flac".parse::<Audio>().unwrap();
    audio.album = "qux".parse().unwrap();
    audio.artist = "baz".parse().unwrap();
    audio.channels = 2;
    audio.disc = 1;
    audio.discs = 2;
    audio.sample_bits = Some(16);
    audio.sample_rate = 44100;
    audio.samples = 66150;
    audio.title = "bar".parse().unwrap();
    audio.track = 3;
    audio.tracks = 4;

    assert_eq!(
      Item::info(&audio, "bob".into()),
      Info::Map(vec![
        (
          "filename".into(),
          Info::Link {
            text: "foo.flac".into(),
            url: "bob".into(),
          },
        ),
        ("title".into(), Info::Value("bar".into())),
        ("artist".into(), Info::Value("baz".into())),
        ("album".into(), Info::Value("qux".into())),
        ("disc".into(), Info::Value("1 of 2".into())),
        ("track".into(), Info::Value("3 of 4".into())),
        ("duration".into(), Info::Value("0:01".into())),
        ("type".into(), Info::Value("FLAC".into())),
        ("sample bits".into(), Info::Value("16-bit".into())),
        ("sample rate".into(), Info::Value("44.1 kHz".into())),
        ("channels".into(), Info::Value("2".into())),
        ("compression mode".into(), Info::Value("lossless".into())),
        ("samples".into(), Info::Value("66150".into())),
      ]),
    );

    let mut audio = "foo.mp3".parse::<Audio>().unwrap();
    audio.album = "qux".parse().unwrap();
    audio.artist = "baz".parse().unwrap();
    audio.channels = 2;
    audio.disc = 1;
    audio.discs = 2;
    audio.sample_rate = 44100;
    audio.samples = 66150;
    audio.title = "bar".parse().unwrap();
    audio.track = 3;
    audio.tracks = 4;

    assert_eq!(
      Item::info(&audio, "bob".into()),
      Info::Map(vec![
        (
          "filename".into(),
          Info::Link {
            text: "foo.mp3".into(),
            url: "bob".into(),
          },
        ),
        ("title".into(), Info::Value("bar".into())),
        ("artist".into(), Info::Value("baz".into())),
        ("album".into(), Info::Value("qux".into())),
        ("disc".into(), Info::Value("1 of 2".into())),
        ("track".into(), Info::Value("3 of 4".into())),
        ("duration".into(), Info::Value("0:01".into())),
        ("type".into(), Info::Value("MP3".into())),
        ("sample rate".into(), Info::Value("44.1 kHz".into())),
        ("channels".into(), Info::Value("2".into())),
        ("compression mode".into(), Info::Value("lossy".into())),
        ("samples".into(), Info::Value("66150".into())),
      ]),
    );
  }

  #[test]
  fn populate_flac_err() {
    fn err(bytes: &[u8]) -> Error {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.flac"), bytes).unwrap();

      let mut audio = "foo.flac".parse::<Audio>().unwrap();

      audio.populate(&root).unwrap_err()
    }

    assert_matches!(err(b"foo"), Error::FlacDecode { .. });

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
      Error::FlacSampleCountUnknown { .. },
    );
  }

  #[test]
  fn populate_flac_ok() {
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
    assert_eq!(audio.sample_bits, Some(16));
    assert_eq!(audio.sample_rate, 44100);
    assert_eq!(audio.samples, 66150);
    assert_eq!(audio.title.as_str(), "bar");
    assert_eq!(audio.track, 3);
    assert_eq!(audio.tracks, 4);
  }

  #[test]
  fn populate_mp3_err() {
    fn err(bytes: &[u8]) -> Error {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.mp3"), bytes).unwrap();

      let mut audio = "foo.mp3".parse::<Audio>().unwrap();

      audio.populate(&root).unwrap_err()
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
  fn populate_mp3_ok() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.mp3"),
      mp3(
        &["TALB=qux", "TIT2=bar", "TPE1=baz", "TPOS=1/2", "TRCK=3/4"],
        2,
      ),
    )
    .unwrap();

    let mut audio = "foo.mp3".parse::<Audio>().unwrap();
    audio.populate(&root).unwrap();

    assert_eq!(audio.album.as_str(), "qux");
    assert_eq!(audio.artist.as_str(), "baz");
    assert_eq!(audio.channels, 2);
    assert_eq!(audio.disc, 1);
    assert_eq!(audio.discs, 2);
    assert_eq!(audio.sample_bits, None);
    assert_eq!(audio.sample_rate, 44100);
    assert_eq!(audio.samples, 2304);
    assert_eq!(audio.title.as_str(), "bar");
    assert_eq!(audio.track, 3);
    assert_eq!(audio.tracks, 4);
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Audio>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","channels":0,"disc":0,"discs":0,"filename":"foo.flac","sample_rate":0,"samples":0,"title":"","track":0,"tracks":0,"type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&"foo.mp3".parse::<Audio>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","channels":0,"disc":0,"discs":0,"filename":"foo.mp3","sample_rate":0,"samples":0,"title":"","track":0,"tracks":0,"type":"mp3"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Audio {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 8,
        disc: 3,
        discs: 4,
        filename: "foo.flac".parse().unwrap(),
        sample_bits: Some(7),
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
