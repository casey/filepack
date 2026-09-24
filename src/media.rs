use super::*;

#[derive(Clone, Debug, Decode, Encode, EnumDiscriminants, IntoStaticStr, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
#[strum(serialize_all = "kebab-case")]
#[strum_discriminants(
  derive(Display, Ord, PartialOrd),
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
  pub(crate) fn info(&self, builder: InfoBuilder, identifier: PackageIdentifier) -> InfoBuilder {
    fn format<T: Content>(builder: InfoBuilder, items: &[Item<T>]) -> InfoBuilder {
      let formats = Item::formats(items);
      builder.when(!formats.is_empty(), |builder| {
        builder.list(
          "format",
          formats
            .into_iter()
            .map(|format| Info::Value(OrUnknown(format).to_string())),
        )
      })
    }

    builder
      .link("media", self.name(), format!("/package/{identifier}/media"))
      .with(|builder| match self {
        Self::Audio { items } => format(
          builder
            .value("tracks", items.len())
            .value("duration", DisplayDuration(Audio::sum_durations(items))),
          items,
        ),
        Self::Image { items } => format(builder.value("images", items.len()), items),
        Self::Video { items } => format(
          builder
            .value("videos", items.len())
            .value("duration", DisplayDuration(Video::sum_durations(items))),
          items,
        ),
        Self::Web => builder,
      })
  }

  pub(crate) fn item(&self, i: usize) -> Option<&dyn MediaItem> {
    match self {
      Self::Audio { items } => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Image { items } => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Video { items } => items.get(i).map(|item| item as &dyn MediaItem),
      Self::Web => unreachable!(),
    }
  }

  pub(crate) fn item_count(&self) -> usize {
    match self {
      Self::Audio { items } => items.len(),
      Self::Image { items } => items.len(),
      Self::Video { items } => items.len(),
      Self::Web => unreachable!(),
    }
  }

  fn item_url(&self, identifier: PackageIdentifier, item: usize) -> Option<String> {
    (item < self.item_count()).then(|| format!("/package/{identifier}/item/{}", Ordinal(item)))
  }

  pub(crate) fn items<'a>(&'a self) -> Box<dyn Iterator<Item = &dyn MediaItem> + 'a> {
    match self {
      Self::Audio { items } => Box::new(items.iter().map(|item| item as &dyn MediaItem)),
      Self::Image { items } => Box::new(items.iter().map(|item| item as &dyn MediaItem)),
      Self::Video { items } => Box::new(items.iter().map(|item| item as &dyn MediaItem)),
      Self::Web => unreachable!(),
    }
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn next_item_url(&self, identifier: PackageIdentifier, item: usize) -> Option<String> {
    self.item_url(identifier, item.checked_add(1)?)
  }

  pub(crate) fn prev_item_url(&self, identifier: PackageIdentifier, item: usize) -> Option<String> {
    self.item_url(identifier, item.checked_sub(1)?)
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

    for identifier in [
      PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
      PackageIdentifier::Number(1),
    ] {
      assert_eq!(
        media.next_item_url(identifier, 0),
        Some(format!("/package/{identifier}/item/2")),
      );
      assert_eq!(media.next_item_url(identifier, 1), None);
      assert_eq!(media.prev_item_url(identifier, 0), None);
      assert_eq!(
        media.prev_item_url(identifier, 1),
        Some(format!("/package/{identifier}/item/1")),
      );
    }
  }
}
