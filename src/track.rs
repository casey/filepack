use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Track {
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

impl Track {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  pub(crate) fn check_content(&self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    match self.ty {
      AudioType::Flac => self.check_content_flac(&path),
    }
  }

  fn check_content_flac(&self, path: &Utf8Path) -> Result {
    let reader = FlacReader::open(path).context(error::TrackDecode { path })?;

    let Streaminfo {
      channels,
      sample_bits,
      sample_rate,
      samples,
    } = Self::streaminfo(&reader, path)?;

    ensure! {
      channels == self.channels,
      error::TrackChannelsMismatch {
        actual: channels,
        expected: self.channels,
        path,
      },
    }

    ensure! {
      sample_bits == self.sample_bits,
      error::TrackSampleBitsMismatch {
        actual: sample_bits,
        expected: self.sample_bits,
        path,
      },
    }

    ensure! {
      sample_rate == self.sample_rate,
      error::TrackSampleRateMismatch {
        actual: sample_rate,
        expected: self.sample_rate,
        path,
      },
    }

    ensure! {
      samples == self.samples,
      error::TrackSampleCountMismatch {
        actual: samples,
        expected: self.samples,
        path,
      },
    }

    Ok(())
  }

  pub(crate) fn check_positions(tracks: &[Track]) -> Result<(), TrackError> {
    let Some(first) = tracks.first() else {
      return Ok(());
    };

    let discs = first.discs;

    let mut expected_disc = 1;
    let mut expected_track = 1;
    let mut disc_tracks = 0;

    for track in tracks {
      ensure! {
        track.discs == discs,
        track_error::DiscTotalMismatch {
          actual: track.discs,
          expected: discs,
          filename: track.filename.clone(),
        },
      }

      ensure! {
        track.disc == expected_disc && track.track == expected_track,
        track_error::PositionMismatch {
          disc: track.disc,
          expected_disc,
          expected_track,
          filename: track.filename.clone(),
          track: track.track,
        },
      }

      ensure! {
        track.disc <= discs,
        track_error::DiscNumberExceedsTotal {
          filename: track.filename.clone(),
          number: track.disc,
          total: discs,
        },
      }

      if expected_track == 1 {
        disc_tracks = track.tracks;
      } else {
        ensure! {
          track.tracks == disc_tracks,
          track_error::TotalMismatch {
            actual: track.tracks,
            disc: expected_disc,
            expected: disc_tracks,
            filename: track.filename.clone(),
          },
        }
      }

      ensure! {
        track.track <= disc_tracks,
        track_error::NumberExceedsTotal {
          filename: track.filename.clone(),
          number: track.track,
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
      track_error::Missing {
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

  pub(crate) fn format(&self) -> AudioFormat {
    AudioFormat {
      channels: self.channels,
      sample_bits: self.sample_bits,
      sample_rate: self.sample_rate,
      ty: self.ty,
    }
  }

  pub(crate) fn formats(tracks: &[Track]) -> Vec<AudioFormat> {
    let mut formats = Vec::new();

    for track in tracks {
      let format = track.format();
      if !formats.contains(&format) {
        formats.push(format);
      }
    }

    formats
  }

  fn number_tag(reader: &FlacReader<fs::File>, path: &Utf8Path, tag: &'static str) -> Result<u64> {
    Self::tag(reader, path, tag)?
      .parse()
      .context(error::TrackTagInteger { path, tag })
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    match self.ty {
      AudioType::Flac => self.populate_flac(&path),
    }
  }

  fn populate_flac(&mut self, path: &Utf8Path) -> Result {
    let reader = FlacReader::open(path).context(error::TrackDecode { path })?;

    let Streaminfo {
      channels,
      sample_bits,
      sample_rate,
      samples,
    } = Self::streaminfo(&reader, path)?;

    self.album = Self::text_tag(&reader, path, "album")?;
    self.artist = Self::text_tag(&reader, path, "artist")?;
    self.channels = channels;
    self.disc = Self::number_tag(&reader, path, "discnumber")?;
    self.discs = Self::number_tag(&reader, path, "disctotal")?;
    self.sample_bits = sample_bits;
    self.sample_rate = sample_rate;
    self.samples = samples;
    self.title = Self::text_tag(&reader, path, "title")?;
    self.track = Self::number_tag(&reader, path, "tracknumber")?;
    self.tracks = Self::number_tag(&reader, path, "tracktotal")?;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }

  fn streaminfo(reader: &FlacReader<fs::File>, path: &Utf8Path) -> Result<Streaminfo> {
    let streaminfo = reader.streaminfo();

    let samples = streaminfo
      .samples
      .context(error::TrackSampleCountUnknown { path })?;

    Ok(Streaminfo {
      channels: streaminfo.channels.into(),
      sample_bits: streaminfo.bits_per_sample.into(),
      sample_rate: streaminfo.sample_rate.into(),
      samples,
    })
  }

  pub(crate) fn sum_durations(tracks: &[Track]) -> Duration {
    tracks.iter().fold(Duration::ZERO, |sum, track| {
      sum.saturating_add(track.duration())
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
      .context(error::TrackTagMissing { path, tag })?;

    ensure! {
      values.next().is_none(),
      error::TrackTagMultiple { path, tag },
    }

    ensure! {
      !value.is_empty(),
      error::TrackTagEmpty { path, tag },
    }

    Ok(value)
  }

  fn text_tag(reader: &FlacReader<fs::File>, path: &Utf8Path, tag: &'static str) -> Result<Text> {
    Self::tag(reader, path, tag)?
      .parse()
      .context(error::TrackTagInvalid { path, tag })
  }
}

impl FromStr for Track {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check_content() {
    #[track_caller]
    fn case(track: &Track) -> Result {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.flac"), flac(&[], 44100)).unwrap();

      track.check_content(&root)
    }

    let mut track = "foo.flac".parse::<Track>().unwrap();
    track.channels = 2;
    track.sample_bits = 16;
    track.sample_rate = 44100;
    track.samples = 44100;

    case(&track).unwrap();

    track.samples = 1;

    assert_matches_regex!(
      case(&track).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has 44100 samples but metadata sample count is 1$",
    );

    track.samples = 44100;
    track.sample_rate = 22050;

    assert_matches_regex!(
      case(&track).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has sample rate 44100 but metadata sample rate is 22050$",
    );

    track.sample_rate = 44100;
    track.sample_bits = 24;

    assert_matches_regex!(
      case(&track).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has 16 bits per sample but metadata sample bits is 24$",
    );

    track.sample_bits = 16;
    track.channels = 1;

    assert_matches_regex!(
      case(&track).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has 2 channels but metadata channel count is 1$",
    );
  }

  #[test]
  fn check_positions() {
    #[track_caller]
    fn case(positions: &[(u64, u64, u64, u64)], expected: Result<(), TrackError>) {
      let tracks = positions
        .iter()
        .enumerate()
        .map(|(i, (disc, discs, track, tracks))| {
          let mut t = format!("{i}.flac").parse::<Track>().unwrap();
          t.disc = *disc;
          t.discs = *discs;
          t.track = *track;
          t.tracks = *tracks;
          t
        })
        .collect::<Vec<Track>>();

      assert_eq!(Track::check_positions(&tracks), expected);
    }

    case(&[], Ok(()));

    case(&[(1, 1, 1, 1)], Ok(()));

    case(&[(1, 2, 1, 2), (1, 2, 2, 2), (2, 2, 1, 1)], Ok(()));

    case(
      &[(1, 1, 2, 2), (1, 1, 1, 2)],
      Err(TrackError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        filename: "0.flac".parse().unwrap(),
        track: 2,
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 1, 2)],
      Err(TrackError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        filename: "1.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 1, 3), (1, 1, 3, 3)],
      Err(TrackError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        filename: "1.flac".parse().unwrap(),
        track: 3,
      }),
    );

    case(
      &[(1, 1, 1, 2)],
      Err(TrackError::Missing { disc: 1, track: 2 }),
    );

    case(
      &[(1, 2, 1, 1)],
      Err(TrackError::Missing { disc: 2, track: 1 }),
    );

    case(
      &[(1, 2, 1, 1), (2, 1, 1, 1)],
      Err(TrackError::DiscTotalMismatch {
        actual: 1,
        expected: 2,
        filename: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 2, 3)],
      Err(TrackError::TotalMismatch {
        actual: 3,
        disc: 1,
        expected: 2,
        filename: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 1), (2, 1, 1, 1)],
      Err(TrackError::DiscNumberExceedsTotal {
        filename: "1.flac".parse().unwrap(),
        number: 2,
        total: 1,
      }),
    );

    case(
      &[(1, 0, 1, 1)],
      Err(TrackError::DiscNumberExceedsTotal {
        filename: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(1, 1, 1, 0)],
      Err(TrackError::NumberExceedsTotal {
        filename: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(0, 1, 1, 1)],
      Err(TrackError::PositionMismatch {
        disc: 0,
        expected_disc: 1,
        expected_track: 1,
        filename: "0.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 0, 1)],
      Err(TrackError::PositionMismatch {
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
      let mut track = "foo.flac".parse::<Track>().unwrap();
      track.sample_rate = sample_rate;
      track.samples = samples;
      assert_eq!(track.duration(), expected);
    }

    case(0, 0, Duration::ZERO);
    case(44100, 44100, Duration::from_secs(1));
    case(66150, 44100, Duration::from_millis(1500));
    case(u64::MAX, u64::MAX - 1, Duration::new(1, 0));
  }

  #[test]
  fn encoding() {
    assert_cbor(
      "foo.flac".parse::<Track>().unwrap(),
      "ad006001600200030004000568666f6f2e666c616306000700080009600a000b000c00",
    );

    assert_cbor(
      Track {
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
    let mut track = "foo.flac".parse::<Track>().unwrap();
    track.channels = 2;
    track.sample_bits = 16;
    track.sample_rate = 44100;

    assert_eq!(
      track.format(),
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
    let mut foo = "foo.flac".parse::<Track>().unwrap();
    foo.channels = 2;
    foo.sample_bits = 16;
    foo.sample_rate = 44100;

    let mut bar = foo.clone();
    bar.sample_bits = 24;

    assert_eq!(
      Track::formats(&[foo.clone(), bar.clone(), foo.clone()]),
      [foo.format(), bar.format()],
    );
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: ComponentError) {
      assert_eq!(s.parse::<Track>().unwrap_err(), expected);
    }

    assert_eq!(
      "foo.flac".parse::<Track>().unwrap(),
      Track {
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

      let mut track = "foo.flac".parse::<Track>().unwrap();

      track.populate(&root).unwrap_err()
    }

    assert_matches!(err(b"foo"), Error::TrackDecode { .. });

    assert_matches!(
      err(&flac(&[], 44100)),
      Error::TrackTagMissing { tag: "album", .. },
    );

    assert_matches!(
      err(&flac(&["ALBUM=qux", "TITLE=bar"], 44100)),
      Error::TrackTagMissing { tag: "artist", .. },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ARTIST=baz", "DISCNUMBER=1", "DISCTOTAL=1"],
        44100,
      )),
      Error::TrackTagMissing { tag: "title", .. },
    );

    assert_matches!(
      err(&flac(
        &["ALBUM=qux", "ALBUM=quux", "ARTIST=baz", "TITLE=bar"],
        44100,
      )),
      Error::TrackTagMultiple { tag: "album", .. },
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
      Error::TrackTagEmpty { tag: "title", .. },
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
      Error::TrackTagInvalid {
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
      Error::TrackTagMissing {
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
      Error::TrackTagInteger {
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
      Error::TrackTagInteger {
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
      Error::TrackSampleCountUnknown { .. },
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

    let mut track = "foo.flac".parse::<Track>().unwrap();
    track.populate(&root).unwrap();

    assert_eq!(track.album.as_str(), "qux");
    assert_eq!(track.artist.as_str(), "baz");
    assert_eq!(track.channels, 2);
    assert_eq!(track.disc, 1);
    assert_eq!(track.discs, 2);
    assert_eq!(track.sample_bits, 16);
    assert_eq!(track.sample_rate, 44100);
    assert_eq!(track.samples, 66150);
    assert_eq!(track.title.as_str(), "bar");
    assert_eq!(track.track, 3);
    assert_eq!(track.tracks, 4);
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Track>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","channels":0,"disc":0,"discs":0,"filename":"foo.flac","sample_bits":0,"sample_rate":0,"samples":0,"title":"","track":0,"tracks":0,"type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
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
