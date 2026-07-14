use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum TrackError {
  #[snafu(display("track `{filename}` has disc number {number} but disc total is {total}"))]
  DiscNumberExceedsTotal {
    filename: ComponentBuf,
    number: u64,
    total: u64,
  },
  #[snafu(display(
    "track `{filename}` has disc total {actual} but first track has disc total {expected}"
  ))]
  DiscTotalMismatch {
    actual: u64,
    expected: u64,
    filename: ComponentBuf,
  },
  #[snafu(display("package is missing disc {disc} track {track}"))]
  Missing { disc: u64, track: u64 },
  #[snafu(display("track `{filename}` has track number {number} but track total is {total}"))]
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
