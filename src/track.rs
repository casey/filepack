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
  pub(crate) title: Text,
  #[n(4)]
  #[serde(rename = "type")]
  pub(crate) ty: AudioType,
}

impl Track {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.filename.as_path()
  }

  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    let path = root.join(self.as_path());

    match self.ty {
      AudioType::Flac => self.populate_flac(&path),
    }
  }

  fn populate_flac(&mut self, path: &Utf8Path) -> Result {
    let reader = FlacReader::open(path).context(error::TrackDecode { path })?;

    self.album = Self::tag(&reader, path, "album")?;
    self.artist = Self::tag(&reader, path, "artist")?;
    self.title = Self::tag(&reader, path, "title")?;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
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
      title: Text::new(),
      ty,
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
      "a5006001600268666f6f2e666c616303600400",
    );

    assert_cbor(
      Track {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        filename: "foo.flac".parse().unwrap(),
        title: "bar".parse().unwrap(),
        ty: AudioType::Flac,
      },
      "a50063717578016362617a0268666f6f2e666c616303636261720400",
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
      case(&flac(&[])),
      Error::TrackTagMissing { tag: "album", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "TITLE=bar"])),
      Error::TrackTagMissing { tag: "artist", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "ARTIST=baz"])),
      Error::TrackTagMissing { tag: "title", .. },
    );

    assert_matches!(
      case(&flac(&[
        "ALBUM=qux",
        "ALBUM=quux",
        "ARTIST=baz",
        "TITLE=bar"
      ])),
      Error::TrackTagMultiple { tag: "album", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "ARTIST=baz", "TITLE="])),
      Error::TrackTagEmpty { tag: "title", .. },
    );

    assert_matches!(
      case(&flac(&["ALBUM=qux", "ARTIST=baz", "TITLE=foo\tbar"])),
      Error::TrackTagInvalid {
        source: TextError::Control { character: '\t' },
        tag: "title",
        ..
      },
    );
  }

  #[test]
  fn populate_ok() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.flac"),
      flac(&["ALBUM=qux", "ARTIST=baz", "TITLE=bar"]),
    )
    .unwrap();

    let mut track = "foo.flac".parse::<Track>().unwrap();
    track.populate(&root).unwrap();

    assert_eq!(track.album.as_str(), "qux");
    assert_eq!(track.artist.as_str(), "baz");
    assert_eq!(track.title.as_str(), "bar");
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Track>().unwrap()).unwrap(),
      r#"{"album":"","artist":"","filename":"foo.flac","title":"","type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        filename: "foo.flac".parse().unwrap(),
        title: "bar".parse().unwrap(),
        ty: AudioType::Flac,
      })
      .unwrap(),
      r#"{"album":"qux","artist":"baz","filename":"foo.flac","title":"bar","type":"flac"}"#,
    );
  }
}
