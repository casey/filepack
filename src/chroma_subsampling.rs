use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, Json, PartialEq, Serialize)]
pub(crate) enum ChromaSubsampling {
  /// Luma only, no chroma.
  #[n(0)]
  #[serde(rename = "4:0:0")]
  #[strum(serialize = "4:0:0")]
  Yuv400,
  /// Chroma at half horizontal and half vertical resolution.
  #[n(1)]
  #[serde(rename = "4:2:0")]
  #[strum(serialize = "4:2:0")]
  Yuv420,
  /// Chroma at half horizontal and full vertical resolution.
  #[n(2)]
  #[serde(rename = "4:2:2")]
  #[strum(serialize = "4:2:2")]
  Yuv422,
  /// Chroma at full horizontal and half vertical resolution.
  #[n(3)]
  #[serde(rename = "4:4:0")]
  #[strum(serialize = "4:4:0")]
  Yuv440,
  /// Chroma at full resolution.
  #[n(4)]
  #[serde(rename = "4:4:4")]
  #[strum(serialize = "4:4:4")]
  Yuv444,
}
