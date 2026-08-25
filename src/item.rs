use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, PartialEq, Serialize)]
pub(crate) struct Item<T> {
  #[n(0)]
  pub(crate) content: T,
  #[n(1)]
  pub(crate) title: Option<Text>,
}

impl<T: Content> Item<T> {
  pub(crate) fn display_title(&self) -> &str {
    match &self.title {
      Some(title) => title,
      None => self.content.path().as_ref(),
    }
  }
}

impl Item<Audio> {
  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    self.title = Some(self.content.populate(root)?);
    Ok(())
  }
}

impl Item<Video> {
  pub(crate) fn populate(&mut self, root: &Utf8Path) -> Result {
    self.title = self.content.populate(root)?;
    Ok(())
  }
}

impl<T: Content> MediaItem for Item<T> {
  fn info(&self, url: String) -> Info {
    self
      .content
      .info(
        InfoBuilder::new()
          .link("path", self.content.path(), url)
          .optional("title", self.title.as_ref()),
      )
      .build()
  }

  fn path(&self) -> &RelativePath {
    self.content.path()
  }

  fn resource_type(&self) -> ResourceType {
    self.content.resource_type()
  }
}

impl<'de, T> Deserialize<'de> for Item<T>
where
  T: FromStr,
  T::Err: Display,
{
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    String::deserialize(deserializer)?
      .parse()
      .map_err(serde::de::Error::custom)
  }
}

impl<T: FromStr> FromStr for Item<T> {
  type Err = T::Err;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self {
      content: s.parse()?,
      title: None,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display_title() {
    let mut item = "foo.png".parse::<Item<Image>>().unwrap();
    assert_eq!(item.display_title(), "foo.png");
    item.title = Some("bar".parse().unwrap());
    assert_eq!(item.display_title(), "bar");
  }

  #[test]
  fn encoding() {
    assert_encoding(Item {
      content: "foo.png".parse::<Image>().unwrap(),
      title: Some("bar".parse().unwrap()),
    });
  }

  #[test]
  fn from_str() {
    assert_eq!(
      "foo.png".parse::<Item<Image>>().unwrap(),
      Item {
        content: "foo.png".parse().unwrap(),
        title: None,
      },
    );

    assert_eq!(
      "foo".parse::<Item<Image>>().unwrap_err(),
      PathError::Extension {
        extensions: ImageType::EXTENSIONS,
      },
    );
  }

  #[test]
  fn info() {
    let mut item = "foo.png".parse::<Item<Image>>().unwrap();

    assert_eq!(
      MediaItem::info(&item, "bar".into()),
      InfoBuilder::new()
        .link("path", "foo.png", "bar".into())
        .value("type", "PNG")
        .value("dimensions", "0×0")
        .value("orientation", "0°")
        .value("color type", "RGB")
        .value("bit depth", "0-bit")
        .value("alpha", "false")
        .build(),
    );

    item.title = Some("baz".parse().unwrap());

    assert_eq!(
      MediaItem::info(&item, "bar".into()),
      InfoBuilder::new()
        .link("path", "foo.png", "bar".into())
        .value("title", "baz")
        .value("type", "PNG")
        .value("dimensions", "0×0")
        .value("orientation", "0°")
        .value("color type", "RGB")
        .value("bit depth", "0-bit")
        .value("alpha", "false")
        .build(),
    );
  }

  #[test]
  fn populate_audio() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.flac"),
      FlacBuilder::new()
        .tag("ALBUM", "qux")
        .tag("ARTIST", "baz")
        .tag("DISCNUMBER", "1")
        .tag("DISCTOTAL", "1")
        .tag("TITLE", "bar")
        .tag("TRACKNUMBER", "1")
        .tag("TRACKTOTAL", "1")
        .samples(1)
        .build(),
    )
    .unwrap();

    let mut item = "foo.flac".parse::<Item<Audio>>().unwrap();
    item.populate(&root).unwrap();
    assert_eq!(item.title, Some("bar".parse().unwrap()));
  }

  #[test]
  fn populate_video() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.mp4"),
      Mp4Builder::new().video_track(2, 1).name("bar").build(),
    )
    .unwrap();

    let mut item = "foo.mp4".parse::<Item<Video>>().unwrap();
    item.populate(&root).unwrap();
    assert_eq!(item.title, Some("bar".parse().unwrap()));
  }
}
