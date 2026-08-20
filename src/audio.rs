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
  pub(crate) path: RelativePath,
  #[n(6)]
  pub(crate) sample_bits: Option<u64>,
  #[n(7)]
  pub(crate) sample_rate: u64,
  #[n(8)]
  pub(crate) samples: u64,
  #[n(9)]
  pub(crate) size: u64,
  #[n(10)]
  pub(crate) title: Text,
  #[n(11)]
  pub(crate) track: u64,
  #[n(12)]
  pub(crate) tracks: u64,
  #[n(13)]
  #[serde(rename = "type")]
  pub(crate) ty: AudioType,
}

impl Audio {
  pub(crate) fn check_positions(tracks: &[Audio]) -> Result<(), AudioPositionError> {
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
        audio_position_error::DiscTotalMismatch {
          actual: audio.discs,
          expected: discs,
          path: audio.path.clone(),
        },
      }

      ensure! {
        audio.disc == expected_disc && audio.track == expected_track,
        audio_position_error::PositionMismatch {
          disc: audio.disc,
          expected_disc,
          expected_track,
          path: audio.path.clone(),
          track: audio.track,
        },
      }

      ensure! {
        audio.disc <= discs,
        audio_position_error::DiscNumberExceedsTotal {
          path: audio.path.clone(),
          number: audio.disc,
          total: discs,
        },
      }

      if expected_track == 1 {
        disc_tracks = audio.tracks;
      } else {
        ensure! {
          audio.tracks == disc_tracks,
          audio_position_error::TotalMismatch {
            actual: audio.tracks,
            disc: expected_disc,
            expected: disc_tracks,
            path: audio.path.clone(),
          },
        }
      }

      ensure! {
        audio.track <= disc_tracks,
        audio_position_error::NumberExceedsTotal {
          path: audio.path.clone(),
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
      audio_position_error::Missing {
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

  pub(crate) fn formats(tracks: &[Audio]) -> Vec<AudioType> {
    let mut formats = Vec::new();

    for audio in tracks {
      if !formats.contains(&audio.ty) {
        formats.push(audio.ty);
      }
    }

    formats
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let path = root.join(&self.path);

    let metadata = match self.ty {
      AudioType::Flac => FlacDecoder::read(&path)?,
      AudioType::Mp3 => Mp3Decoder::read(&path)?,
    };

    let AudioMetadata {
      album,
      artist,
      channels,
      disc,
      discs,
      sample_bits,
      sample_rate,
      samples,
      size,
      title,
      track,
      tracks,
    } = metadata;

    self.album = album;
    self.artist = artist;
    self.channels = channels;
    self.disc = disc;
    self.discs = discs;
    self.sample_bits = sample_bits;
    self.sample_rate = sample_rate;
    self.samples = samples;
    self.size = size;
    self.title = title;
    self.track = track;
    self.tracks = tracks;

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

  pub(crate) fn tag<'a>(
    mut values: impl Iterator<Item = &'a str>,
    tag: &'static str,
  ) -> Result<&'a str, AudioError> {
    let value = values.next().context(audio_error::TagMissing { tag })?;

    ensure! {
      values.next().is_none(),
      audio_error::TagMultiple { tag },
    }

    ensure! {
      !value.is_empty(),
      audio_error::TagEmpty { tag },
    }

    Ok(value)
  }
}

impl FromStr for Audio {
  type Err = PathError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let path = s.parse::<RelativePath>()?;

    let Some(ty) = path.extension().and_then(AudioType::from_extension) else {
      return Err(PathError::Extension {
        extensions: AudioType::EXTENSIONS,
      });
    };

    Ok(Self {
      album: Text::new(),
      artist: Text::new(),
      channels: 0,
      disc: 0,
      discs: 0,
      path,
      sample_bits: None,
      sample_rate: 0,
      samples: 0,
      size: 0,
      title: Text::new(),
      track: 0,
      tracks: 0,
      ty,
    })
  }
}

impl Item for Audio {
  fn info(&self, url: String) -> Info {
    InfoBuilder::new()
      .link("path", &self.path, url)
      .value("title", &self.title)
      .value("artist", &self.artist)
      .value("album", &self.album)
      .value("disc", format!("{} of {}", self.disc, self.discs))
      .value("track", format!("{} of {}", self.track, self.tracks))
      .value("duration", DisplayDuration(self.duration()))
      .value("type", self.ty)
      .optional(
        "sample bits",
        self
          .sample_bits
          .map(|sample_bits| format!("{sample_bits}-bit")),
      )
      .value("sample rate", DisplaySampleRate(self.sample_rate))
      .optional(
        "bit rate",
        DisplayBitrate::new(
          u64::try_from(self.duration().as_millis()).unwrap_or(u64::MAX),
          self.size,
        ),
      )
      .value("channels", self.channels)
      .value(
        "compression mode",
        match self.ty {
          AudioType::Flac => "lossless",
          AudioType::Mp3 => "lossy",
        },
      )
      .value("samples", self.samples)
      .build()
  }

  fn path(&self) -> &RelativePath {
    &self.path
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
    fn case(positions: &[(u64, u64, u64, u64)], expected: Result<(), AudioPositionError>) {
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
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        path: "0.flac".parse().unwrap(),
        track: 2,
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 1, 2)],
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        path: "1.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 1, 3), (1, 1, 3, 3)],
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 2,
        path: "1.flac".parse().unwrap(),
        track: 3,
      }),
    );

    case(
      &[(1, 1, 1, 2)],
      Err(AudioPositionError::Missing { disc: 1, track: 2 }),
    );

    case(
      &[(1, 2, 1, 1)],
      Err(AudioPositionError::Missing { disc: 2, track: 1 }),
    );

    case(
      &[(1, 2, 1, 1), (2, 1, 1, 1)],
      Err(AudioPositionError::DiscTotalMismatch {
        actual: 1,
        expected: 2,
        path: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 2), (1, 1, 2, 3)],
      Err(AudioPositionError::TotalMismatch {
        actual: 3,
        disc: 1,
        expected: 2,
        path: "1.flac".parse().unwrap(),
      }),
    );

    case(
      &[(1, 1, 1, 1), (2, 1, 1, 1)],
      Err(AudioPositionError::DiscNumberExceedsTotal {
        path: "1.flac".parse().unwrap(),
        number: 2,
        total: 1,
      }),
    );

    case(
      &[(1, 0, 1, 1)],
      Err(AudioPositionError::DiscNumberExceedsTotal {
        path: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(1, 1, 1, 0)],
      Err(AudioPositionError::NumberExceedsTotal {
        path: "0.flac".parse().unwrap(),
        number: 1,
        total: 0,
      }),
    );

    case(
      &[(0, 1, 1, 1)],
      Err(AudioPositionError::PositionMismatch {
        disc: 0,
        expected_disc: 1,
        expected_track: 1,
        path: "0.flac".parse().unwrap(),
        track: 1,
      }),
    );

    case(
      &[(1, 1, 0, 1)],
      Err(AudioPositionError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        path: "0.flac".parse().unwrap(),
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
    fn case(s: &str, expected: PathError) {
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
        path: "foo.flac".parse().unwrap(),
        sample_bits: None,
        sample_rate: 0,
        samples: 0,
        size: 0,
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
        path: "foo.mp3".parse().unwrap(),
        sample_bits: None,
        sample_rate: 0,
        samples: 0,
        size: 0,
        title: Text::new(),
        track: 0,
        tracks: 0,
        ty: AudioType::Mp3,
      },
    );

    assert_eq!(
      "foo/bar.flac".parse::<Audio>().unwrap().path,
      "foo/bar.flac".parse::<RelativePath>().unwrap(),
    );

    case(
      "foo.wav",
      PathError::Extension {
        extensions: &["flac", "mp3"],
      },
    );
    case(
      "foo",
      PathError::Extension {
        extensions: &["flac", "mp3"],
      },
    );
    case("", PathError::Empty);
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
    audio.size = 750;
    audio.title = "bar".parse().unwrap();
    audio.track = 3;
    audio.tracks = 4;

    assert_eq!(
      Item::info(&audio, "bob".into()),
      InfoBuilder::new()
        .link("path", "foo.flac", "bob".into())
        .value("title", "bar")
        .value("artist", "baz")
        .value("album", "qux")
        .value("disc", "1 of 2")
        .value("track", "3 of 4")
        .value("duration", "0:01")
        .value("type", "FLAC")
        .value("sample bits", "16-bit")
        .value("sample rate", "44.1 kHz")
        .value("bit rate", "4 kbit/s")
        .value("channels", "2")
        .value("compression mode", "lossless")
        .value("samples", "66150")
        .build(),
    );

    let mut audio = "foo.mp3".parse::<Audio>().unwrap();
    audio.album = "qux".parse().unwrap();
    audio.artist = "baz".parse().unwrap();
    audio.channels = 2;
    audio.disc = 1;
    audio.discs = 2;
    audio.sample_rate = 44100;
    audio.samples = 66150;
    audio.size = 750;
    audio.title = "bar".parse().unwrap();
    audio.track = 3;
    audio.tracks = 4;

    assert_eq!(
      Item::info(&audio, "bob".into()),
      InfoBuilder::new()
        .link("path", "foo.mp3", "bob".into())
        .value("title", "bar")
        .value("artist", "baz")
        .value("album", "qux")
        .value("disc", "1 of 2")
        .value("track", "3 of 4")
        .value("duration", "0:01")
        .value("type", "MP3")
        .value("sample rate", "44.1 kHz")
        .value("bit rate", "4 kbit/s")
        .value("channels", "2")
        .value("compression mode", "lossy")
        .value("samples", "66150")
        .build(),
    );

    let mut audio = "foo.flac".parse::<Audio>().unwrap();
    audio.size = 750;

    assert_eq!(
      Item::info(&audio, "bob".into()),
      InfoBuilder::new()
        .link("path", "foo.flac", "bob".into())
        .value("title", "")
        .value("artist", "")
        .value("album", "")
        .value("disc", "0 of 0")
        .value("track", "0 of 0")
        .value("duration", "0:00")
        .value("type", "FLAC")
        .value("sample rate", "0 kHz")
        .value("channels", "0")
        .value("compression mode", "lossless")
        .value("samples", "0")
        .build(),
    );
  }

  #[test]
  fn populate() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.flac"),
      FlacBuilder::new()
        .tag("ALBUM", "qux")
        .tag("ARTIST", "baz")
        .tag("DISCNUMBER", "1")
        .tag("DISCTOTAL", "2")
        .tag("TITLE", "bar")
        .tag("TRACKNUMBER", "3")
        .tag("TRACKTOTAL", "4")
        .samples(66150)
        .build(),
    )
    .unwrap();

    std::fs::write(
      root.join("foo.mp3"),
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

    let mut audio = "foo.flac".parse::<Audio>().unwrap();
    audio.populate(&root).unwrap();

    assert_eq!(
      audio,
      Audio {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        disc: 1,
        discs: 2,
        path: "foo.flac".parse().unwrap(),
        sample_bits: Some(16),
        sample_rate: 44100,
        samples: 66150,
        size: 1024,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
        ty: AudioType::Flac,
      },
    );

    let mut audio = "foo.mp3".parse::<Audio>().unwrap();
    audio.populate(&root).unwrap();

    assert_eq!(
      audio,
      Audio {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        disc: 1,
        discs: 2,
        path: "foo.mp3".parse().unwrap(),
        sample_bits: None,
        sample_rate: 44100,
        samples: 2304,
        size: 834,
        title: "bar".parse().unwrap(),
        track: 3,
        tracks: 4,
        ty: AudioType::Mp3,
      },
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Audio>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","channels":0,"disc":0,"discs":0,"path":"foo.flac","sample_rate":0,"samples":0,"size":0,"title":"","track":0,"tracks":0,"type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&"foo.mp3".parse::<Audio>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","channels":0,"disc":0,"discs":0,"path":"foo.mp3","sample_rate":0,"samples":0,"size":0,"title":"","track":0,"tracks":0,"type":"mp3"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Audio {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 8,
        disc: 3,
        discs: 4,
        path: "foo.flac".parse().unwrap(),
        sample_bits: Some(7),
        sample_rate: 1,
        samples: 2,
        size: 9,
        title: "bar".parse().unwrap(),
        track: 5,
        tracks: 6,
        ty: AudioType::Flac,
      })
      .unwrap(),
      r#"{"album":"qux","artist":"baz","channels":8,"disc":3,"discs":4,"path":"foo.flac","sample_bits":7,"sample_rate":1,"samples":2,"size":9,"title":"bar","track":5,"tracks":6,"type":"flac"}"#,
    );
  }
}
