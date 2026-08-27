use super::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "type")]
pub(crate) enum Media {
  Audio { items: Vec<RelativePath> },
  Image { items: Vec<RelativePath> },
  Video { items: Vec<Video> },
  Web,
}

impl Media {
  pub(crate) fn load(self, root: &Utf8Path, bar: &ProgressBar) -> Result<crate::Media> {
    fn load<T: Content>(
      root: &Utf8Path,
      items: Vec<RelativePath>,
      bar: &ProgressBar,
    ) -> Result<Vec<Item<T>>> {
      items
        .into_iter()
        .map(|path| {
          let item = T::load(root, path)?;
          bar.inc(1);
          Ok(item)
        })
        .collect()
    }

    Ok(match self {
      Self::Audio { items } => crate::Media::Audio {
        items: load(root, items, bar)?,
      },
      Self::Image { items } => crate::Media::Image {
        items: load(root, items, bar)?,
      },
      Self::Video { items } => crate::Media::Video {
        items: items
          .into_iter()
          .map(|Video { cover, path }| {
            let mut item = crate::Video::load(root, path)?;
            bar.inc(1);
            if let Some(cover) = cover {
              item.content.cover = Some(Image::load(root, cover)?.content);
              bar.inc(1);
            }
            Ok(item)
          })
          .collect::<Result<Vec<Item<crate::Video>>>>()?,
      },
      Self::Web => crate::Media::Web,
    })
  }
}
