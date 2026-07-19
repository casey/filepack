use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VideoFormat {
  pub(crate) audio_codec: Codec,
  pub(crate) ty: VideoType,
  pub(crate) video_codec: Codec,
}

impl Display for VideoFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {} {}", self.ty, self.video_codec, self.audio_codec)
  }
}
