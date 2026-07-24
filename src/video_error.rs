use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum VideoError {
  #[snafu(display("track {track} has unsupported audio codec `{codec}`"))]
  AudioCodecUnsupported { codec: String, track: usize },
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
  #[snafu(display("video has duration {actual}ms but metadata has duration {expected}ms"))]
  DurationMismatch { actual: u64, expected: u64 },
  #[snafu(display("missing duration"))]
  DurationMissing,
  #[snafu(display("duration overflow"))]
  DurationOverflow,
  #[snafu(display("zero timescale"))]
  TimescaleZero,
  #[snafu(display("unsupported timestamp scale {timestamp_scale}"))]
  TimestampScale { timestamp_scale: u64 },
  #[snafu(display(
    "video has {} but metadata has {}",
    Count::new(*actual, "track"),
    Count::new(*expected, "track"),
  ))]
  TrackCountMismatch { actual: usize, expected: usize },
  #[snafu(display("video track {index} `{actual}` doesn't match metadata track `{expected}`"))]
  TrackMismatch {
    actual: Track,
    expected: Track,
    index: usize,
  },
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
}
