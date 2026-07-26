use super::*;

pub(crate) struct DisplayBitrate {
  pub(crate) duration: u64,
  pub(crate) size: u64,
}

impl Display for DisplayBitrate {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let duration = u128::from(self.duration);

    let bits_per_second = (u128::from(self.size) * 8 * 1000 + duration / 2) / duration;

    let bits_per_second = u64::try_from(bits_per_second).unwrap_or(u64::MAX);

    let formatter = SizeFormatter::new(
      bits_per_second,
      FormatSizeOptions::from(DECIMAL)
        .base_unit(BaseUnit::Bit)
        .decimal_places(1),
    );

    write!(f, "{formatter}/s")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(size: u64, duration: u64, expected: &str) {
      assert_eq!(DisplayBitrate { duration, size }.to_string(), expected);
    }

    case(0, 1000, "0 bits/s");
    case(1, 1000, "8 bits/s");
    case(1, 3000, "3 bits/s");
    case(125, 1000, "1 kbit/s");
    case(1500, 10_000, "1.2 kbit/s");
    case(125_000, 1000, "1 Mbit/s");
    case(437_500, 1000, "3.5 Mbit/s");
    case(125_000_000, 1000, "1 Gbit/s");
  }
}
