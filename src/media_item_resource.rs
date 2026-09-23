use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MediaItemResource {
  Original,
  Placeholder,
  PlaceholderThumbnail,
  Thumbnail,
}
