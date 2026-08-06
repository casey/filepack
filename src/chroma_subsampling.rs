use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
pub(crate) enum ChromaSubsampling {
  #[n(0)]
  #[serde(rename = "4:0:0")]
  #[strum(serialize = "4:0:0")]
  Yuv400,
  #[n(1)]
  #[serde(rename = "4:2:0")]
  #[strum(serialize = "4:2:0")]
  Yuv420,
  #[n(2)]
  #[serde(rename = "4:2:2")]
  #[strum(serialize = "4:2:2")]
  Yuv422,
  #[n(3)]
  #[serde(rename = "4:4:0")]
  #[strum(serialize = "4:4:0")]
  Yuv440,
  #[n(4)]
  #[serde(rename = "4:4:4")]
  #[strum(serialize = "4:4:4")]
  Yuv444,
}
