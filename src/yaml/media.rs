use super::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "type")]
pub(crate) enum Media {
  Audio { items: Vec<Audio> },
  Image { items: Vec<Image> },
  Video { items: Vec<Video> },
  Web,
}

impl Media {
  pub(crate) fn items_missing(&self) -> bool {
    match self {
      Self::Audio { items } => items.is_empty(),
      Self::Image { items } => items.is_empty(),
      Self::Video { items } => items.is_empty(),
      Self::Web => false,
    }
  }

  pub(crate) fn load(self, root: &Utf8Path, bar: &ProgressBar) -> Result<crate::Media> {
    Ok(match self {
      Self::Audio { items } => crate::Media::Audio {
        items: items
          .into_iter()
          .map(|Audio { path }| {
            let item = crate::Audio::load(root, path)?;
            bar.inc(1);
            Ok(item)
          })
          .collect::<Result<Vec<Item<crate::Audio>>>>()?,
      },
      Self::Image { items } => crate::Media::Image {
        items: items
          .into_iter()
          .map(|Image { path }| {
            let item = crate::Image::load(root, path)?;
            bar.inc(1);
            Ok(item)
          })
          .collect::<Result<Vec<Item<crate::Image>>>>()?,
      },
      Self::Video { items } => crate::Media::Video {
        items: items
          .into_iter()
          .map(|Video { placeholder, path }| {
            let mut item = crate::Video::load(root, path)?;
            bar.inc(1);
            if let Some(placeholder) = placeholder {
              item.content.placeholder = Some(crate::Image::load(root, placeholder)?.content);
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn items_missing() {
    #[track_caller]
    fn case(media: Media, expected: bool) {
      assert_eq!(media.items_missing(), expected);
    }

    case(Media::Audio { items: Vec::new() }, true);
    case(Media::Image { items: Vec::new() }, true);
    case(Media::Video { items: Vec::new() }, true);
    case(
      Media::Image {
        items: vec![Image {
          path: "foo.png".parse().unwrap(),
        }],
      },
      false,
    );
    case(Media::Web, false);
  }
}
