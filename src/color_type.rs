use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Display, Encode, PartialEq, Serialize)]
pub(crate) enum ColorType {
  #[n(0)]
  #[serde(rename = "cmyk")]
  #[strum(serialize = "CMYK")]
  Cmyk,
  #[n(1)]
  #[serde(rename = "grayscale")]
  #[strum(serialize = "grayscale")]
  Grayscale,
  #[n(2)]
  #[serde(rename = "indexed")]
  #[strum(serialize = "indexed")]
  Indexed,
  #[default]
  #[n(3)]
  #[serde(rename = "rgb")]
  #[strum(serialize = "RGB")]
  Rgb,
}
