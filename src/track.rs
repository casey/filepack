use super::*;

#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Track {
  #[n(0)]
  pub(crate) album: Text,
  #[n(1)]
  pub(crate) artist: Text,
  #[n(2)]
  pub(crate) filename: ComponentBuf,
  #[n(3)]
  pub(crate) sample_count: u64,
  #[n(4)]
  pub(crate) sample_rate: u64,
  #[n(5)]
  pub(crate) title: Text,
  #[n(6)]
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
      sample_count,
      sample_rate,
    } = Self::streaminfo(&reader, path)?;

    ensure! {
      sample_count == self.sample_count,
      error::TrackSampleCountMismatch {
        actual: sample_count,
        expected: self.sample_count,
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

    Ok(())
  }

  pub(crate) fn duration(&self) -> Duration {
    if self.sample_rate == 0 {
      return Duration::ZERO;
    }

    let subsecond = u128::from(self.sample_count % self.sample_rate);

    Duration::new(
      self.sample_count / self.sample_rate,
      u32::try_from(subsecond * 1_000_000_000 / u128::from(self.sample_rate)).unwrap(),
    )
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
      sample_count,
      sample_rate,
    } = Self::streaminfo(&reader, path)?;

    self.album = Self::tag(&reader, path, "album")?;
    self.artist = Self::tag(&reader, path, "artist")?;
    self.sample_count = sample_count;
    self.sample_rate = sample_rate;
    self.title = Self::tag(&reader, path, "title")?;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }

  fn streaminfo(reader: &FlacReader<fs::File>, path: &Utf8Path) -> Result<Streaminfo> {
    let streaminfo = reader.streaminfo();

    let sample_count = streaminfo
      .samples
      .context(error::TrackSampleCountUnknown { path })?;

    Ok(Streaminfo {
      sample_count,
      sample_rate: streaminfo.sample_rate.into(),
    })
  }

  fn tag(reader: &FlacReader<fs::File>, path: &Utf8Path, tag: &'static str) -> Result<Text> {
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

    value.parse().context(error::TrackTagInvalid { path, tag })
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
      filename,
      sample_count: 0,
      sample_rate: 0,
      title: Text::new(),
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
    track.sample_count = 44100;
    track.sample_rate = 44100;

    case(&track).unwrap();

    track.sample_count = 1;

    assert_matches_regex!(
      case(&track).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has 44100 samples but metadata sample count is 1$",
    );

    track.sample_count = 44100;
    track.sample_rate = 22050;

    assert_matches_regex!(
      case(&track).unwrap_err().to_string(),
      r"^track `.*foo\.flac` has sample rate 44100 but metadata sample rate is 22050$",
    );
  }

  #[test]
  fn duration() {
    #[track_caller]
    fn case(sample_count: u64, sample_rate: u64, expected: Duration) {
      let mut track = "foo.flac".parse::<Track>().unwrap();
      track.sample_count = sample_count;
      track.sample_rate = sample_rate;
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
      "a7006001600268666f6f2e666c61630300040005600600",
    );

    assert_cbor(
      Track {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        filename: "foo.flac".parse().unwrap(),
        sample_count: 2,
        sample_rate: 1,
        title: "bar".parse().unwrap(),
        ty: AudioType::Flac,
      },
      "a70063717578016362617a0268666f6f2e666c61630302040105636261720600",
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
        filename: "foo.flac".parse().unwrap(),
        sample_count: 0,
        sample_rate: 0,
        title: Text::new(),
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
    #[track_caller]
    fn case(bytes: &[u8]) -> Error {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.flac"), bytes).unwrap();

      let mut track = "foo.flac".parse::<Track>().unwrap();

      track.populate(&root).unwrap_err()
    }

    assert_matches!(case(b"foo"), Error::TrackDecode { .. });

    assert_matches!(
      case(&flac(&[], 44100)),
      Error::TrackTagMissing { tag: "album", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "TITLE=bar"], 44100)),
      Error::TrackTagMissing { tag: "artist", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "ARTIST=baz"], 44100)),
      Error::TrackTagMissing { tag: "title", .. },
    );

    assert_matches!(
      case(&flac(
        &["ALBUM=qux", "ALBUM=quux", "ARTIST=baz", "TITLE=bar"],
        44100,
      )),
      Error::TrackTagMultiple { tag: "album", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "ARTIST=baz", "TITLE="], 44100)),
      Error::TrackTagEmpty { tag: "title", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "ARTIST=baz", "TITLE=foo\tbar"], 44100)),
      Error::TrackTagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "title",
        ..
      },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "ARTIST=baz", "TITLE=bar"], 0)),
      Error::TrackSampleCountUnknown { .. },
    );
  }

  #[test]
  fn populate_ok() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.flac"),
      flac(&["ALBUM=qux", "ARTIST=baz", "TITLE=bar"], 66150),
    )
    .unwrap();

    let mut track = "foo.flac".parse::<Track>().unwrap();
    track.populate(&root).unwrap();

    assert_eq!(track.album.as_str(), "qux");
    assert_eq!(track.artist.as_str(), "baz");
    assert_eq!(track.sample_count, 66150);
    assert_eq!(track.sample_rate, 44100);
    assert_eq!(track.title.as_str(), "bar");
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Track>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","filename":"foo.flac","sample_count":0,"sample_rate":0,"title":"","type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        filename: "foo.flac".parse().unwrap(),
        sample_count: 2,
        sample_rate: 1,
        title: "bar".parse().unwrap(),
        ty: AudioType::Flac,
      })
      .unwrap(),
      r#"{"album":"qux","artist":"baz","filename":"foo.flac","sample_count":2,"sample_rate":1,"title":"bar","type":"flac"}"#,
    );
  }
}
