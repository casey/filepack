use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum Codec {
  #[n(0)]
  Aac,
  #[n(1)]
  #[strum(serialize = "H.264")]
  H264,
  #[n(2)]
  Mp3,
  #[n(3)]
  #[strum(serialize = "Opus")]
  Opus,
  #[n(4)]
  #[strum(serialize = "Vorbis")]
  Vorbis,
  #[n(5)]
  Vp8,
  #[n(6)]
  Vp9,
}
