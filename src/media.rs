use super::*;

#[derive(Clone, Debug, Decode, Encode, EnumDiscriminants, IntoStaticStr, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
#[strum(serialize_all = "kebab-case")]
#[strum_discriminants(
  derive(Display),
  name(MediaType),
  strum(serialize_all = "kebab-case"),
  vis(pub)
)]
pub(crate) enum Media {
  #[n(0)]
  Audio {
    #[n(0)]
    items: Vec<Item<Audio>>,
  },
  #[n(1)]
  Image {
    #[n(0)]
    items: Vec<Item<Image>>,
  },
  #[n(2)]
  Video {
    #[n(0)]
    items: Vec<Item<Video>>,
  },
  #[n(3)]
  Web,
}

impl Media {
  pub(crate) fn item(&self, i: usize) -> Option<&dyn MediaItem> {
    match self {
      Self::Audio { items } => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Image { items } => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Video { items } => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Web => unreachable!(),
    }
  }

  fn item_url(&self, fingerprint: Fingerprint, item: usize) -> Option<String> {
    (item < self.items()).then(|| format!("/package/{fingerprint}/item/{}", Ordinal(item)))
  }

  pub(crate) fn items(&self) -> usize {
    match self {
      Self::Audio { items } => items.len(),
      Self::Image { items } => items.len(),
      Self::Video { items } => items.len(),
      Self::Web => unreachable!(),
    }
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn next_item_url(&self, fingerprint: Fingerprint, item: usize) -> Option<String> {
    self.item_url(fingerprint, item.checked_add(1)?)
  }

  pub(crate) fn prev_item_url(&self, fingerprint: Fingerprint, item: usize) -> Option<String> {
    self.item_url(fingerprint, item.checked_sub(1)?)
  }

  pub(crate) fn ty(&self) -> MediaType {
    self.discriminant()
  }
}

impl MediaType {
  pub(crate) fn has_items(self) -> bool {
    match self {
      Self::Audio | Self::Image | Self::Video => true,
      Self::Web => false,
    }
  }

  pub(crate) fn item_noun(self) -> &'static str {
    match self {
      Self::Audio => "track",
      Self::Image => "image",
      Self::Video => "video",
      Self::Web => unreachable!(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn item_url() {
    let media = Media::Image {
      items: vec![Item::test("foo.png"), Item::test("bar.png")],
    };

    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();

    assert_eq!(
      media.next_item_url(fingerprint, 0),
      Some(format!("/package/{fingerprint}/item/2")),
    );
    assert_eq!(media.next_item_url(fingerprint, 1), None);
    assert_eq!(media.prev_item_url(fingerprint, 0), None);
    assert_eq!(
      media.prev_item_url(fingerprint, 1),
      Some(format!("/package/{fingerprint}/item/1")),
    );
  }
}
