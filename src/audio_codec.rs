use super::*;

/// Audio file codec.
#[derive(Clone, Copy, Debug, Decode, Display, Encode, Json, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum AudioCodec {
  /// [Advanced Audio Coding](https://en.wikipedia.org/wiki/Advanced_Audio_Coding).
  #[n(0)]
  Aac,
  /// [Free Lossless Audio Codec](https://en.wikipedia.org/wiki/FLAC).
  #[n(1)]
  Flac,
  /// [MPEG-1 Audio Layer III](https://en.wikipedia.org/wiki/MP3).
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
