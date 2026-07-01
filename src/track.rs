use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Track {
  #[n(0)]
  pub(crate) bits_per_sample: Byte,
  #[n(1)]
  pub(crate) channel_count: u64,
  #[n(2)]
  pub(crate) disc_number: u64,
  #[n(3)]
  pub(crate) filename: ComponentBuf,
  #[n(4)]
  pub(crate) sample_count: u64,
  #[n(5)]
  pub(crate) sample_rate: u64,
  #[n(6)]
  pub(crate) title: Option<String>,
  #[n(7)]
  pub(crate) track_number: u64,
  #[n(8)]
  #[serde(rename = "type")]
  pub(crate) ty: AudioType,
}

impl Track {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  fn number<R: io::Read>(reader: &FlacReader<R>, path: &Utf8Path, tag: &str) -> Result<u64> {
    let mut values = reader.get_tag(tag);

    let value = values
      .next()
      .context(error::TrackNumberMissing { path, tag })?;

    ensure! {
      values.next().is_none(),
      error::TrackNumberMultiple { path, tag },
    }

    value
      .parse::<u64>()
      .ok()
      .filter(|_| re::NUMBER.is_match(value))
      .context(error::TrackNumberInvalid { path, tag, value })
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

    let streaminfo = reader.streaminfo();

    self.title = title;
    self.track_number = Self::number(&reader, path, "TRACKNUMBER")?;
    self.disc_number = Self::number(&reader, path, "DISCNUMBER")?;
    self.channel_count = streaminfo.channels.into();
    self.sample_rate = streaminfo.sample_rate.into();
    self.sample_count = streaminfo
      .samples
      .context(error::TrackSampleCountMissing { path })?;
    self.bits_per_sample = Byte(streaminfo.bits_per_sample.try_into().unwrap());

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
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
      title: None,
      ty,
      track_number: 0,
      disc_number: 0,
      channel_count: 0,
      sample_rate: 0,
      sample_count: 0,
      bits_per_sample: Byte(0),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(
      "foo.flac".parse::<Track>().unwrap(),
      "a80000010002000368666f6f2e666c61630400050007000800",
    );

    assert_cbor(
      Track {
        filename: "foo.flac".parse().unwrap(),
        title: Some("bar".into()),
        ty: AudioType::Flac,
        track_number: 1,
        disc_number: 2,
        channel_count: 3,
        sample_rate: 4,
        sample_count: 5,
        bits_per_sample: Byte(6),
      },
      "a90006010302020368666f6f2e666c616304050504066362617207010800",
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
        title: None,
        ty: AudioType::Flac,
        track_number: 0,
        disc_number: 0,
        channel_count: 0,
        sample_rate: 0,
        sample_count: 0,
        bits_per_sample: Byte(0),
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
      case(&flac(&["TITLE=bar", "TRACKNUMBER=7", "DISCNUMBER=3"])).unwrap(),
      Track {
        filename: "foo.flac".parse().unwrap(),
        title: Some("bar".into()),
        ty: AudioType::Flac,
        track_number: 7,
        disc_number: 3,
        channel_count: 2,
        sample_rate: 44100,
        sample_count: 44100,
        bits_per_sample: Byte(16),
      },
    );

    assert_eq!(
      case(&flac(&["TRACKNUMBER=1", "DISCNUMBER=1"]))
        .unwrap()
        .title,
      None,
    );

    assert_eq!(
      case(&flac(&["ARTIST=bar", "TRACKNUMBER=1", "DISCNUMBER=1"]))
        .unwrap()
        .title,
      None,
    );

    assert_matches_regex!(
      case(b"foo").unwrap_err().to_string(),
      r"^failed to decode FLAC track `.*foo\.flac`$",
    );

    assert_matches_regex!(
      case(&flac(&["TITLE=bar", "TITLE=baz"]))
        .unwrap_err()
        .to_string(),
      r"^FLAC track `.*foo\.flac` has multiple titles$",
    );

    assert_matches_regex!(
      case(&flac(&["TITLE="])).unwrap_err().to_string(),
      r"^FLAC track `.*foo\.flac` has empty title$",
    );

    assert_matches_regex!(
      case(&flac(&["DISCNUMBER=1"])).unwrap_err().to_string(),
      r"^FLAC track `.*foo\.flac` is missing `TRACKNUMBER` comment$",
    );

    assert_matches_regex!(
      case(&flac(&["TRACKNUMBER=1", "TRACKNUMBER=2", "DISCNUMBER=1"]))
        .unwrap_err()
        .to_string(),
      r"^FLAC track `.*foo\.flac` has multiple `TRACKNUMBER` comments$",
    );

    assert_matches_regex!(
      case(&flac(&["TRACKNUMBER=01", "DISCNUMBER=1"]))
        .unwrap_err()
        .to_string(),
      r"^FLAC track `.*foo\.flac` has invalid `TRACKNUMBER` comment `01`$",
    );

    assert_matches_regex!(
      case(&flac(&["TRACKNUMBER=1/12", "DISCNUMBER=1"]))
        .unwrap_err()
        .to_string(),
      r"^FLAC track `.*foo\.flac` has invalid `TRACKNUMBER` comment `1/12`$",
    );

    let mut bytes = flac(&["TRACKNUMBER=1", "DISCNUMBER=1"]);
    bytes[22..26].copy_from_slice(&[0; 4]);
    assert_matches_regex!(
      case(&bytes).unwrap_err().to_string(),
      r"^FLAC track `.*foo\.flac` has unknown sample count$",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Track>().unwrap()).unwrap(),
      r#"{"bits_per_sample":0,"channel_count":0,"disc_number":0,"filename":"foo.flac","sample_count":0,"sample_rate":0,"track_number":0,"type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        filename: "foo.flac".parse().unwrap(),
        title: Some("bar".into()),
        ty: AudioType::Flac,
        track_number: 1,
        disc_number: 2,
        channel_count: 3,
        sample_rate: 4,
        sample_count: 5,
        bits_per_sample: Byte(6),
      })
      .unwrap(),
      r#"{"bits_per_sample":6,"channel_count":3,"disc_number":2,"filename":"foo.flac","sample_count":5,"sample_rate":4,"title":"bar","track_number":1,"type":"flac"}"#,
    );
  }
}
