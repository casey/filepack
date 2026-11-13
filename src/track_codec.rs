use super::*;

/// Codec of a video file track.
#[derive(Clone, Copy, Debug, Decode, Display, Encode, Json, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum TrackCodec {
  /// [Advanced Audio Coding](https://en.wikipedia.org/wiki/Advanced_Audio_Coding).
  #[n(0)]
  Aac,
  /// [Advanced Video Coding](https://en.wikipedia.org/wiki/Advanced_Video_Coding).
  #[n(1)]
  #[strum(serialize = "H.264")]
  H264,
  /// [MPEG-1 Audio Layer III](https://en.wikipedia.org/wiki/MP3).
  #[n(2)]
  Mp3,
  /// [Opus](https://en.wikipedia.org/wiki/Opus_(audio_format)).
  #[n(3)]
  #[strum(serialize = "Opus")]
  Opus,
  /// [Vorbis](https://en.wikipedia.org/wiki/Vorbis).
  #[n(4)]
  #[strum(serialize = "Vorbis")]
  Vorbis,
  /// [VP8](https://en.wikipedia.org/wiki/VP8).
  #[n(5)]
  Vp8,
  /// [VP9](https://en.wikipedia.org/wiki/VP9).
  #[n(6)]
  Vp9,
}
