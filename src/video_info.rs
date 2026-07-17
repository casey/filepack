use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct VideoInfo {
  pub(crate) audio_codec: AudioCodec,
  pub(crate) dimensions: Dimensions,
  pub(crate) video_codec: VideoCodec,
}
