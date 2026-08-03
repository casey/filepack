use super::*;

#[derive(
  Clone, Debug, Decode, Deserialize, Encode, EnumDiscriminants, IntoStaticStr, PartialEq, Serialize,
)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "type")]
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
    tracks: Vec<Audio>,
  },
  #[n(1)]
  Image {
    #[n(0)]
    images: Vec<Image>,
  },
  #[n(2)]
  Video {
    #[n(0)]
    videos: Vec<Video>,
  },
  #[n(3)]
  Web,
}

impl Media {
  pub(crate) fn item(&self, i: usize) -> Option<&dyn Item> {
    match self {
      Self::Audio { tracks } => tracks.get(i).map(|item| item as &dyn Item),
      Self::Image { images } => images.get(i).map(|item| item as &dyn Item),
      Self::Video { videos } => videos.get(i).map(|item| item as &dyn Item),
      Self::Web => unreachable!(),
    }
  }

  pub(crate) fn items(&self) -> usize {
    match self {
      Self::Audio { tracks } => tracks.len(),
      Self::Image { images } => images.len(),
      Self::Video { videos } => videos.len(),
      Self::Web => unreachable!(),
    }
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
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
