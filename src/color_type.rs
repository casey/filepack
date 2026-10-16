use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Display, Encode, Json, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ColorType {
  #[n(0)]
  #[strum(serialize = "CMYK")]
  Cmyk,
  #[n(1)]
  #[strum(serialize = "grayscale")]
  Grayscale,
  #[n(2)]
  #[strum(serialize = "indexed")]
  Indexed,
  #[default]
  #[n(3)]
  #[strum(serialize = "RGB")]
  Rgb,
}
