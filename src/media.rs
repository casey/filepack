use super::*;

/// Package media content metadata.
#[derive(
  Clone, Debug, Decode, Encode, EnumDiscriminants, IntoStaticStr, Json, PartialEq, Serialize,
)]
#[serde(rename_all = "snake_case", tag = "type")]
#[strum(serialize_all = "kebab-case")]
#[strum_discriminants(
  derive(Deserialize, Display, EnumIter, Ord, PartialOrd, Serialize),
  name(MediaType),
  serde(rename_all = "kebab-case"),
  strum(serialize_all = "kebab-case"),
  vis(pub)
)]
pub(crate) enum Media {
  /// Collection of audio items.
  #[n(0)]
  Audio {
    /// Metadata describing the audio items.
    #[n(1)]
    items: Vec<Item<Audio>>,
  },
  /// Collection of document items.
  #[n(1)]
  Document {
    /// Metadata describing the document items.
    #[n(1)]
    items: Vec<Item<Document>>,
  },
  /// Collection of image items.
  #[n(2)]
  Image {
    /// Metadata describing the image items.
    #[n(1)]
    items: Vec<Item<Image>>,
  },
  /// Collection of video items.
  #[n(3)]
  Video {
    /// Metadata describing the video items.
    #[n(1)]
    items: Vec<Item<Video>>,
  },
  /// A web site, with `static/index.html` as its entry point.
  #[n(4)]
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
            .map(|format| Info::Value(format.to_string())),
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
        Self::Document { items } => format(builder.value("documents", items.len()), items),
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

  pub(crate) fn items(&self) -> Option<Items> {
    match self {
      Self::Audio { items } => Some(Items::Audio(items)),
      Self::Document { items } => Some(Items::Document(items)),
      Self::Image { items } => Some(Items::Image(items)),
      Self::Video { items } => Some(Items::Video(items)),
      Self::Web => None,
    }
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn ty(&self) -> MediaType {
    self.discriminant()
  }
}
