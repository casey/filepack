use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum VideoError {
  #[snafu(display("audio codec {actual} doesn't match metadata audio codec {expected}"))]
  AudioCodecMismatch {
    actual: AudioCodec,
    expected: AudioCodec,
  },
  #[snafu(display("unsupported audio codec `{codec}`"))]
  AudioCodecUnsupported { codec: String },
  #[snafu(display("no audio track"))]
  AudioTrackMissing,
  #[snafu(display("multiple audio tracks"))]
  AudioTrackMultiple,
  #[snafu(display("failed to decode MP4"))]
  DecodeMp4 { source: mp4parse::Error },
  #[snafu(display("video is {actual} but metadata dimensions are {expected}"))]
  DimensionsMismatch {
    actual: Dimensions,
    expected: Dimensions,
  },
  #[snafu(display("track has missing sample description"))]
  SampleDescriptionMissing,
  #[snafu(display("track has multiple sample descriptions"))]
  SampleDescriptionMultiple,
  #[snafu(display("unsupported track type `{ty}`"))]
  TrackUnsupported { ty: String },
  #[snafu(display("video codec {actual} doesn't match metadata video codec {expected}"))]
  VideoCodecMismatch {
    actual: VideoCodec,
    expected: VideoCodec,
  },
  #[snafu(display("unsupported video codec `{codec}`"))]
  VideoCodecUnsupported { codec: String },
  #[snafu(display("no video track"))]
  VideoTrackMissing,
  #[snafu(display("multiple video tracks"))]
  VideoTrackMultiple,
}
