use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum MediaItemResource {
  Original,
  Placeholder,
  PlaceholderThumbnail,
  Thumbnail,
}
