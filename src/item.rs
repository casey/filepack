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
  pub(crate) fn display_title(&self, index: usize) -> String {
    match &self.title {
      Some(title) => title.to_string(),
      None => format!("{} {}", T::LABEL, Ordinal(index)),
    }
  }

  pub(crate) fn formats(items: &[Self]) -> Vec<T::Type> {
    let mut formats = Vec::new();

    for item in items {
      if !formats.contains(&item.content.ty()) {
        formats.push(item.content.ty());
      }
    }

    formats
  }

  #[cfg(test)]
  pub(crate) fn test(path: &str) -> Self {
    Self {
      content: T::test(path),
      title: None,
    }
  }
}

impl<T: Content> MediaItem for Item<T> {
  fn info(&self, url: String) -> Info {
    self
      .content
      .info(
        InfoBuilder::new()
          .link("file", self.content.path(), url)
          .optional("title", self.title.as_ref()),
      )
      .build()
  }

  fn path(&self) -> &RelativePath {
    self.content.path()
  }

  fn placeholder(&self) -> Option<&Image> {
    self.content.placeholder()
  }

  fn resource_type(&self) -> ResourceType {
    self.content.resource_type()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display_title() {
    let mut item = Item::<Image>::test("foo.png");
    assert_eq!(item.display_title(0), "Image 1");
    item.title = Some("bar".parse().unwrap());
    assert_eq!(item.display_title(0), "bar");
  }

  #[test]
  fn encoding() {
    assert_encoding(Item {
      content: Image::test("foo.png"),
      title: Some("bar".parse().unwrap()),
    });
  }

  #[test]
  fn formats() {
    let items = [
      Item::<Image>::test("foo.png"),
      Item::test("bar.jpg"),
      Item::test("baz.png"),
    ];

    assert_eq!(Item::formats(&items), [ImageType::Png, ImageType::Jpeg]);
  }

  #[test]
  fn info() {
    let mut item = Item::<Image>::test("foo.png");

    assert_eq!(
      MediaItem::info(&item, "bar".into()),
      InfoBuilder::new()
        .link("file", "foo.png", "bar".into())
        .value("type", "PNG")
        .value("dimensions", "1×1")
        .value("orientation", "0°")
        .value("color type", "RGB")
        .value("bit depth", "8-bit")
        .value("alpha", "false")
        .build(),
    );

    item.title = Some("baz".parse().unwrap());

    assert_eq!(
      MediaItem::info(&item, "bar".into()),
      InfoBuilder::new()
        .link("file", "foo.png", "bar".into())
        .value("title", "baz")
        .value("type", "PNG")
        .value("dimensions", "1×1")
        .value("orientation", "0°")
        .value("color type", "RGB")
        .value("bit depth", "8-bit")
        .value("alpha", "false")
        .build(),
    );
  }

  #[test]
  fn load_audio() {
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

    let item = Audio::load(&root, "foo.flac".parse().unwrap()).unwrap();
    assert_eq!(item.title, Some("bar".parse().unwrap()));
  }

  #[test]
  fn load_image() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.png"),
      PngBuilder::new().text("Title", "bar").build(),
    )
    .unwrap();

    let item = Image::load(&root, "foo.png".parse().unwrap()).unwrap();
    assert_eq!(item.title, Some("bar".parse().unwrap()));
  }

  #[test]
  fn load_video() {
    let (_tempdir, root) = tempdir();

    std::fs::write(
      root.join("foo.mp4"),
      Mp4Builder::new().video_track(2, 1).name("bar").build(),
    )
    .unwrap();

    let item = Video::load(&root, "foo.mp4".parse().unwrap()).unwrap();
    assert_eq!(item.title, Some("bar".parse().unwrap()));
  }
}
