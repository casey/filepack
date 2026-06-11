use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, Serialize)]
pub(crate) struct Track {
  #[n(0)]
  pub(crate) filename: ComponentBuf,
  #[n(1)]
  pub(crate) title: Option<String>,
  #[n(2)]
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
    let mut reader = FlacReader::open(path).context(error::TrackDecode { path })?;

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

    let mut blocks = reader.blocks();

    let mut buffer = Vec::new();

    while let Some(block) = blocks
      .read_next_or_eof(buffer)
      .context(error::TrackDecode { path })?
    {
      buffer = block.into_buffer();
    }

    self.title = title;

    Ok(())
  }

  pub(crate) fn resource_type(&self) -> ResourceType {
    self.ty.resource_type()
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
      "a20068666f6f2e666c61630200",
    );

    assert_cbor(
      Track {
        filename: "foo.flac".parse().unwrap(),
        title: Some("bar".into()),
        ty: AudioType::Flac,
      },
      "a30068666f6f2e666c616301636261720200",
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
      case(&flac(&["TITLE=bar"])).unwrap().title,
      Some("bar".into()),
    );

    assert_eq!(case(&flac(&[])).unwrap().title, None);

    assert_eq!(case(&flac(&["ARTIST=bar"])).unwrap().title, None);

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
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&"foo.flac".parse::<Track>().unwrap()).unwrap(),
      r#"{"filename":"foo.flac","type":"flac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        filename: "foo.flac".parse().unwrap(),
        title: Some("bar".into()),
        ty: AudioType::Flac,
      })
      .unwrap(),
      r#"{"filename":"foo.flac","title":"bar","type":"flac"}"#,
    );
  }
}
