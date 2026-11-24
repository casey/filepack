use super::*;

#[derive(Clone, Copy, Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Stem {
  Artwork,
  #[strum(serialize = "COLOPHON")]
  Colophon,
  #[strum(serialize = "LICENSE")]
  License,
  Placeholder,
  #[strum(serialize = "README")]
  Readme,
  Thumbnail,
}
