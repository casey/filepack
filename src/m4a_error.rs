use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum M4aError {
  #[snafu(display("no audio track"))]
  AudioTrackMissing,
  #[snafu(display("multiple audio tracks"))]
  AudioTrackMultiple,
  #[snafu(display("invalid `{ty}` box"))]
  BoxInvalid { ty: &'static str },
  #[snafu(display("missing `{ty}` box"))]
  BoxMissing { ty: &'static str },
  #[snafu(display("invalid box size {size}"))]
  BoxSize { size: u64 },
  #[snafu(display("unsupported audio codec `{codec}`"))]
  CodecUnsupported { codec: String },
  #[snafu(display("unsupported cover art data type {data_type}"))]
  CoverDataType { data_type: u32 },
  #[snafu(display("`{tag}` tag has unsupported data type {data_type}"))]
  DataType { data_type: u32, tag: &'static str },
  #[snafu(display("no audio samples"))]
  Empty,
  #[snafu(display("invalid `{tag}` tag"))]
  PairTag { tag: &'static str },
  #[snafu(display("sample count overflow"))]
  SamplesOverflow,
  #[snafu(display("zero timescale"))]
  TimescaleZero,
  #[snafu(display("track {track} has unsupported track type `{ty}`"))]
  TrackUnsupported { track: usize, ty: &'static str },
  #[snafu(display("truncated box"))]
  Truncated,
}
