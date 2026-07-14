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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(error: TrackError, expected: &str) {
      assert_eq!(error.to_string(), expected);
    }

    case(
      TrackError::DiscNumberExceedsTotal {
        filename: "foo.flac".parse().unwrap(),
        number: 2,
        total: 1,
      },
      "track `foo.flac` has disc number 2 but disc total is 1",
    );

    case(
      TrackError::DiscTotalMismatch {
        actual: 1,
        expected: 2,
        filename: "foo.flac".parse().unwrap(),
      },
      "track `foo.flac` has disc total 1 but first track has disc total 2",
    );

    case(
      TrackError::Missing { disc: 2, track: 1 },
      "package is missing disc 2 track 1",
    );

    case(
      TrackError::NumberExceedsTotal {
        filename: "foo.flac".parse().unwrap(),
        number: 1,
        total: 0,
      },
      "track `foo.flac` has track number 1 but track total is 0",
    );

    case(
      TrackError::PositionMismatch {
        disc: 1,
        expected_disc: 1,
        expected_track: 1,
        filename: "foo.flac".parse().unwrap(),
        track: 2,
      },
      "track `foo.flac` is disc 1 track 2 but expected disc 1 track 1",
    );

    case(
      TrackError::TotalMismatch {
        actual: 3,
        disc: 1,
        expected: 2,
        filename: "foo.flac".parse().unwrap(),
      },
      "track `foo.flac` has track total 3 but disc 1 has track total 2",
    );
  }
}
