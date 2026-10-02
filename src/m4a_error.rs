use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum M4aError {
  #[snafu(display("no audio track"))]
  AudioTrackMissing,
  #[snafu(display("multiple audio tracks"))]
  AudioTrackMultiple,
  #[snafu(display("unsupported audio codec `{codec}`"))]
  CodecUnsupported { codec: String },
  #[snafu(display("unsupported cover art data type {data_type}"))]
  CoverDataType { data_type: u32 },
  #[snafu(display("no audio samples"))]
  Empty,
  #[snafu(transparent)]
  Mp4 { source: Mp4Error },
  #[snafu(display("sample count overflow"))]
  SamplesOverflow,
  #[snafu(display("zero timescale"))]
  TimescaleZero,
  #[snafu(display("track {track} has unsupported track type `{ty}`"))]
  TrackUnsupported { track: usize, ty: &'static str },
}
