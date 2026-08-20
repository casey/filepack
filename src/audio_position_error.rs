use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum AudioPositionError {
  #[snafu(display("track `{path}` disc number {number} exceeds disc total of {total}"))]
  DiscNumberExceedsTotal {
    number: u64,
    path: RelativePath,
    total: u64,
  },
  #[snafu(display(
    "track `{path}` disc total {actual} doesn't match first track disc total {expected}"
  ))]
  DiscTotalMismatch {
    actual: u64,
    expected: u64,
    path: RelativePath,
  },
  #[snafu(display("package is missing disc {disc} track {track}"))]
  Missing { disc: u64, track: u64 },
  #[snafu(display("track `{path}` track number {number} exceeds track total {total}"))]
  NumberExceedsTotal {
    number: u64,
    path: RelativePath,
    total: u64,
  },
  #[snafu(display(
    "track `{path}` is disc {disc} track {track} \
     but expected disc {expected_disc} track {expected_track}"
  ))]
  PositionMismatch {
    disc: u64,
    expected_disc: u64,
    expected_track: u64,
    path: RelativePath,
    track: u64,
  },
  #[snafu(display(
    "track `{path}` has track total {actual} but disc {disc} has track total {expected}"
  ))]
  TotalMismatch {
    actual: u64,
    disc: u64,
    expected: u64,
    path: RelativePath,
  },
}
