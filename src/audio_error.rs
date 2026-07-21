use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum AudioError {
  #[snafu(display("track has {actual} channels but metadata channel count is {expected}"))]
  ChannelsMismatch { actual: u64, expected: u64 },
  #[snafu(display("failed to decode FLAC"))]
  Decode { source: claxon::Error },
  #[snafu(display("track `{filename}` disc number {number} exceeds disc total of {total}"))]
  DiscNumberExceedsTotal {
    filename: ComponentBuf,
    number: u64,
    total: u64,
  },
  #[snafu(display(
    "track `{filename}` disc total {actual} doesn't match first track disc total {expected}"
  ))]
  DiscTotalMismatch {
    actual: u64,
    expected: u64,
    filename: ComponentBuf,
  },
  #[snafu(display("package is missing disc {disc} track {track}"))]
  Missing { disc: u64, track: u64 },
  #[snafu(display("track `{filename}` track number {number} exceeds track total {total}"))]
  NumberExceedsTotal {
    filename: ComponentBuf,
    number: u64,
    total: u64,
  },
  #[snafu(display(
    "track `{filename}` is disc {disc} track {track} \
     but expected disc {expected_disc} track {expected_track}"
  ))]
  PositionMismatch {
    disc: u64,
    expected_disc: u64,
    expected_track: u64,
    filename: ComponentBuf,
    track: u64,
  },
  #[snafu(display("track has {actual} bits per sample but metadata sample bits is {expected}"))]
  SampleBitsMismatch { actual: u64, expected: u64 },
  #[snafu(display("track has {actual} samples but metadata sample count is {expected}"))]
  SampleCountMismatch { actual: u64, expected: u64 },
  #[snafu(display("track has unknown sample count"))]
  SampleCountUnknown,
  #[snafu(display("track has sample rate {actual} but metadata sample rate is {expected}"))]
  SampleRateMismatch { actual: u64, expected: u64 },
  #[snafu(display("track has empty `{tag}` tag"))]
  TagEmpty { tag: &'static str },
  #[snafu(display("track has invalid integer `{tag}` tag"))]
  TagInteger {
    source: ParseIntError,
    tag: &'static str,
  },
  #[snafu(display("track has invalid `{tag}` tag"))]
  TagInvalid {
    source: TextError,
    tag: &'static str,
  },
  #[snafu(display("track is missing `{tag}` tag"))]
  TagMissing { tag: &'static str },
  #[snafu(display("track has multiple `{tag}` tags"))]
  TagMultiple { tag: &'static str },
  #[snafu(display(
    "track `{filename}` has track total {actual} but disc {disc} has track total {expected}"
  ))]
  TotalMismatch {
    actual: u64,
    disc: u64,
    expected: u64,
    filename: ComponentBuf,
  },
}
