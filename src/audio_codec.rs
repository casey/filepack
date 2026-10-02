use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum AudioCodec {
  #[n(0)]
  Aac,
  #[n(1)]
  Flac,
  #[n(2)]
  Mp3,
}

impl AudioCodec {
  pub(crate) fn compression(self) -> Compression {
    match self {
      Self::Flac => Compression::Lossless,
      Self::Aac | Self::Mp3 => Compression::Lossy,
    }
  }
}
