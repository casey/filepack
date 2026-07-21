use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum VideoError {
  #[snafu(display("track {track} has unsupported audio codec `{codec}`"))]
  AudioCodecUnsupported { codec: String, track: usize },
  #[snafu(display("multiple audio tracks"))]
  AudioTrackMultiple,
  #[snafu(display("failed to decode MP4"))]
  DecodeMp4 { source: mp4parse::Error },
  #[snafu(display("failed to decode WebM"))]
  DecodeWebm {
    source: matroska_demuxer::DemuxError,
  },
  #[snafu(display("track {track} has missing sample description"))]
  SampleDescriptionMissing { track: usize },
  #[snafu(display("track {track} has multiple sample descriptions"))]
  SampleDescriptionMultiple { track: usize },
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
  #[snafu(display("expected DocType `webm` but found `{doc_type}`"))]
  WebmDocType { doc_type: String },
}
