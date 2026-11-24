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

impl Stem {
  pub(crate) fn filename<T: Format>(self, format: &Decoded<T>) -> String {
    if let Decoded::Known(format) = format {
      format.filename(&self.to_string())
    } else {
      self.to_string()
    }
  }
}
