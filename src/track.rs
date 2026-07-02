use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Track {
  #[n(0)]
  pub(crate) filename: ComponentBuf,
  #[n(1)]
  pub(crate) sample_count: u64,
  #[n(2)]
  pub(crate) sample_rate: u64,
  #[n(3)]
  pub(crate) title: Option<String>,
  #[n(4)]
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

    let (sample_count, sample_rate) = Self::streaminfo(&reader, path)?;

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
      Duration::ZERO
    } else {
      let subsecond = u128::from(self.sample_count % self.sample_rate);

      Duration::new(
        self.sample_count / self.sample_rate,
        u32::try_from(subsecond * 1_000_000_000 / u128::from(self.sample_rate)).unwrap(),
      )
    }
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    match self.ty {
      AudioType::Flac => self.populate_flac(&path),
    }
  }

  fn populate_flac(&mut self, path: &Utf8Path) -> Result {
    let reader = FlacReader::open(path).context(error::TrackDecode { path })?;

    let title = {
      let mut titles = reader.get_tag("title");

      let title = titles.next();

      ensure! {
        titles.next().is_none(),
        error::TrackTitleMultiple { path },
      }

      if let Some(title) = title {
        ensure! {
          !title.is_empty(),
          error::TrackTitleEmpty { path },
        }

        Some(title.into())
      } else {
        None
      }
    };

    let (sample_count, sample_rate) = Self::streaminfo(&reader, path)?;

    self.sample_count = sample_count;
    self.sample_rate = sample_rate;
    self.title = title;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
  }

  fn streaminfo<R: io::Read>(reader: &FlacReader<R>, path: &Utf8Path) -> Result<(u64, u64)> {
    let streaminfo = reader.streaminfo();

    let sample_count = streaminfo
      .samples
      .context(error::TrackSampleCountUnknown { path })?;

    Ok((sample_count, streaminfo.sample_rate.into()))
  }

  pub(crate) fn title(&self) -> &str {
    self.title.as_deref().unwrap_or("Untitled")
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
      filename,
      sample_count: 0,
      sample_rate: 0,
      title: None,
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
      "a40068666f6f2e666c6163010002000400",
    );

    assert_cbor(
      Track {
        filename: "foo.flac".parse().unwrap(),
        sample_count: 2,
        sample_rate: 1,
        title: Some("bar".into()),
        ty: AudioType::Flac,
      },
      "a50068666f6f2e666c61630102020103636261720400",
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
        filename: "foo.flac".parse().unwrap(),
        sample_count: 0,
        sample_rate: 0,
        title: None,
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
  fn populate() {
    #[track_caller]
    fn case(bytes: &[u8]) -> Result<Track> {
      let (_tempdir, root) = tempdir();

      std::fs::write(root.join("foo.flac"), bytes).unwrap();

      let mut track = "foo.flac".parse::<Track>().unwrap();

      track.populate(&root).map(|()| track)
    }

    assert_eq!(
      case(&flac(&["TITLE=bar"], 44100)).unwrap().title,
      Some("bar".into()),
    );

    assert_eq!(case(&flac(&[], 44100)).unwrap().title, None);

    assert_eq!(case(&flac(&["ARTIST=bar"], 44100)).unwrap().title, None);

    let track = case(&flac(&[], 66150)).unwrap();

    assert_eq!(track.sample_count, 66150);

    assert_eq!(track.sample_rate, 44100);

    assert_matches_regex!(
      case(b"foo").unwrap_err().to_string(),
      r"^failed to decode FLAC track `.*foo\.flac`$",
    );

    assert_matches_regex!(
      case(&flac(&["TITLE=bar", "TITLE=baz"], 44100))
        .unwrap_err()
        .to_string(),
      r"^FLAC track `.*foo\.flac` has multiple titles$",
    );

    assert_matches_regex!(
      case(&flac(&["TITLE="], 44100)).unwrap_err().to_string(),
      r"^FLAC track `.*foo\.flac` has empty title$",
    );

    assert_matches_regex!(
      case(&flac(&[], 0)).unwrap_err().to_string(),
      r"^FLAC track `.*foo\.flac` has unknown sample count$",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Track>().unwrap()).unwrap(),
      r#"{"filename":"foo.flac","sample_count":0,"sample_rate":0,"type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        filename: "foo.flac".parse().unwrap(),
        sample_count: 2,
        sample_rate: 1,
        title: Some("bar".into()),
        ty: AudioType::Flac,
      })
      .unwrap(),
      r#"{"filename":"foo.flac","sample_count":2,"sample_rate":1,"title":"bar","type":"flac"}"#,
    );
  }
}
