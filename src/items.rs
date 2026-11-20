use super::*;

pub(crate) enum Items<'a> {
  Audio(&'a [Item<Audio>]),
  Document(&'a [Item<Document>]),
  Image(&'a [Item<Image>]),
  Video(&'a [Item<Video>]),
}

impl<'a> Items<'a> {
  pub(crate) fn formats(&self) -> Vec<String> {
    fn formats<T: Content>(items: &[Item<T>]) -> Vec<String> {
      Item::formats(items)
        .into_iter()
        .map(ToString::to_string)
        .collect()
    }
    match self {
      Self::Audio(items) => formats(items),
      Self::Document(items) => formats(items),
      Self::Image(items) => formats(items),
      Self::Video(items) => formats(items),
    }
  }

  pub(crate) fn get(&self, i: usize) -> Option<&'a dyn MediaItem> {
    match self {
      Self::Audio(items) => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Document(items) => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Image(items) => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Video(items) => items.get(i).map(|item| item as &dyn MediaItem),
    }
  }

  pub(crate) fn len(&self) -> usize {
    match self {
      Self::Audio(items) => items.len(),
      Self::Document(items) => items.len(),
      Self::Image(items) => items.len(),
      Self::Video(items) => items.len(),
    }
  }

  pub(crate) fn next_url(&self, identifier: PackageIdentifier, item: usize) -> Option<String> {
    self.url(identifier, item.checked_add(1)?)
  }

  pub(crate) fn noun(&self) -> &'static str {
    match self {
      Self::Audio(_) => "track",
      Self::Document(_) => "document",
      Self::Image(_) => "image",
      Self::Video(_) => "video",
    }
  }

  pub(crate) fn prev_url(&self, identifier: PackageIdentifier, item: usize) -> Option<String> {
    self.url(identifier, item.checked_sub(1)?)
  }

  fn url(&self, identifier: PackageIdentifier, item: usize) -> Option<String> {
    (item < self.len()).then(|| format!("/package/{identifier}/item/{}", Ordinal(item)))
  }
}

impl<'a> IntoIterator for Items<'a> {
  type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
  type Item = &'a dyn MediaItem;

  fn into_iter(self) -> Self::IntoIter {
    match self {
      Self::Audio(items) => Box::new(items.iter().map(|item| item as &dyn MediaItem)),
      Self::Document(items) => Box::new(items.iter().map(|item| item as &dyn MediaItem)),
      Self::Image(items) => Box::new(items.iter().map(|item| item as &dyn MediaItem)),
      Self::Video(items) => Box::new(items.iter().map(|item| item as &dyn MediaItem)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn formats() {
    assert_eq!(
      Items::Image(&[
        Item::test("foo.png"),
        Item::test("bar.jpg"),
        Item::test("baz.png"),
      ])
      .formats(),
      ["PNG", "JPEG"],
    );
  }

  #[test]
  fn url() {
    let items = [Item::test("foo.png"), Item::test("bar.png")];

    let items = Items::Image(&items);

    for identifier in [
      PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()),
      PackageIdentifier::Number(0),
    ] {
      assert_eq!(
        items.next_url(identifier, 0),
        Some(format!("/package/{identifier}/item/2")),
      );
      assert_eq!(items.next_url(identifier, 1), None);
      assert_eq!(items.prev_url(identifier, 0), None);
      assert_eq!(
        items.prev_url(identifier, 1),
        Some(format!("/package/{identifier}/item/1")),
      );
    }
  }
}
