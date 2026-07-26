use super::*;

pub(crate) struct DisplayBitsPerPixel {
  pub(crate) pixels: u128,
  pub(crate) size: u64,
}

impl Display for DisplayBitsPerPixel {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let milli = (u128::from(self.size) * 8 * 1000 + self.pixels / 2) / self.pixels;
    write!(f, "{}", DisplayMillis(milli))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(size: u64, pixels: u128, expected: &str) {
      assert_eq!(DisplayBitsPerPixel { pixels, size }.to_string(), expected);
    }

    case(0, 480, "0");
    case(1, 100, "0.08");
    case(81, 8000, "0.081");
    case(1, 3, "2.667");
    case(3, 16, "1.5");
    case(1500, 480, "25");
  }
}
