use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VideoFormat {
  pub(crate) audio_codec: AudioCodec,
  pub(crate) ty: VideoType,
  pub(crate) video_codec: VideoCodec,
}

impl Display for VideoFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {} {}", self.ty, self.video_codec, self.audio_codec)
  }
}
