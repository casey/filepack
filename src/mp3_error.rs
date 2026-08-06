use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Mp3Error {
  #[snafu(display("unsupported bitrate index {index}"))]
  Bitrate { index: u8 },
  #[snafu(display("channel count {actual} does not match first frame channel count {expected}"))]
  ChannelsMismatch { actual: u64, expected: u64 },
  #[snafu(display("no MPEG audio frames"))]
  Empty,
  #[snafu(display("invalid MPEG layer"))]
  LayerInvalid,
  #[snafu(display("unsupported MPEG layer {layer}"))]
  LayerUnsupported { layer: u8 },
  #[snafu(display("invalid sample rate index"))]
  SampleRate,
  #[snafu(display("sample rate {actual} does not match first frame sample rate {expected}"))]
  SampleRateMismatch { actual: u64, expected: u64 },
  #[snafu(display("invalid frame sync at offset {offset}"))]
  Sync { offset: usize },
  #[snafu(display("truncated MPEG frame"))]
  Truncated,
  #[snafu(display("invalid MPEG version"))]
  Version,
}
