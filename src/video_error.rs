use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum VideoError {
  #[snafu(display("track {track} has unsupported audio codec `{codec}`"))]
  AudioCodecUnsupported { codec: String, track: usize },
  #[snafu(display("track {track} has missing audio settings"))]
  AudioSettingsMissing { track: usize },
  #[snafu(display("multiple audio tracks"))]
  AudioTrackMultiple,
  #[snafu(display("failed to decode MP4"))]
  DecodeMp4 { source: re_mp4::Error },
  #[snafu(display("failed to decode WebM"))]
  DecodeWebm {
    source: matroska_demuxer::DemuxError,
  },
  #[snafu(display("expected DocType `webm` but found `{doc_type}`"))]
  DocType { doc_type: String },
  #[snafu(display("invalid duration"))]
  DurationInvalid,
  #[snafu(display("missing duration"))]
  DurationMissing,
  #[snafu(display("duration overflow"))]
  DurationOverflow,
  #[snafu(display("track {track} has unsupported transformation matrix"))]
  MatrixUnsupported { track: usize },
  #[snafu(display("track {track} has invalid sample rate {sample_rate}"))]
  SampleRateInvalid { sample_rate: f64, track: usize },
  #[snafu(display("invalid SPS"))]
  SpsInvalid,
  #[snafu(display("missing SPS"))]
  SpsMissing,
  #[snafu(display("invalid `{tag}` tag"))]
  TagInvalid {
    source: TextError,
    tag: &'static str,
  },
  #[snafu(display("`{tag}` tag is not valid UTF-8"))]
  TagUtf8 {
    source: Utf8Error,
    tag: &'static str,
  },
  #[snafu(display("zero timescale"))]
  TimescaleZero,
  #[snafu(display("unsupported timestamp scale {timestamp_scale}"))]
  TimestampScale { timestamp_scale: u64 },
  #[snafu(display("track {track} has unsupported track type `{ty}`"))]
  TrackUnsupported { track: usize, ty: String },
  #[snafu(display("track {track} has unsupported video codec `{codec}`"))]
  VideoCodecUnsupported { codec: String, track: usize },
  #[snafu(display("track {track} has missing video settings"))]
  VideoSettingsMissing { track: usize },
  #[snafu(display("no video track"))]
  VideoTrackMissing,
  #[snafu(display("multiple video tracks"))]
  VideoTrackMultiple,
  #[snafu(display("invalid VP9 frame header"))]
  Vp9FrameHeaderInvalid,
  #[snafu(display("empty VP9 video track: cannot determine color metadata without frames"))]
  Vp9TrackEmpty,
}
