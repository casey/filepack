use super::*;

pub(crate) struct DisplayFrameRate {
  pub(crate) duration: u64,
  pub(crate) frames: u64,
}

impl Display for DisplayFrameRate {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let duration = u128::from(self.duration);

    let milli_fps = (u128::from(self.frames) * 1_000_000 + duration / 2) / duration;

    let fps = milli_fps / 1000;
    let frac = format!("{:03}", milli_fps % 1000);
    let frac = frac.trim_end_matches('0');

    if frac.is_empty() {
      write!(f, "{fps} fps")
    } else {
      write!(f, "{fps}.{frac} fps")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(frames: u64, duration: u64, expected: &str) {
      assert_eq!(DisplayFrameRate { duration, frames }.to_string(), expected);
    }

    case(0, 1000, "0 fps");
    case(24, 1000, "24 fps");
    case(240, 10000, "24 fps");
    case(24000, 1_001_000, "23.976 fps");
    case(30000, 1_001_000, "29.97 fps");
    case(60000, 1_001_000, "59.94 fps");
    case(120_000, 1_001_000, "119.88 fps");
    case(25, 2000, "12.5 fps");
    case(1, 3000, "0.333 fps");
    case(2, 3000, "0.667 fps");
    case(u64::MAX, 1, "18446744073709551615000 fps");
  }
}
