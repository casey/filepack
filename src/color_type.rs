use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Display, Encode, Json, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ColorType {
  /// Cyan, magenta, yellow, and black.
  #[n(0)]
  #[strum(serialize = "CMYK")]
  Cmyk,
  /// Grayscale.
  #[n(1)]
  #[strum(serialize = "grayscale")]
  Grayscale,
  /// Palette indices.
  #[n(2)]
  #[strum(serialize = "indexed")]
  Indexed,
  /// Red, green, and blue.
  #[default]
  #[n(3)]
  #[strum(serialize = "RGB")]
  Rgb,
}
