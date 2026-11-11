use super::*;

pub(crate) struct DisplayBitsPerPixel {
  pixels: u128,
  size: u64,
}

impl DisplayBitsPerPixel {
  pub(crate) fn new(dimensions: Dimensions, frames: u64, size: u64) -> Option<Self> {
    let pixels = (u128::from(dimensions.width) * u128::from(dimensions.height))
      .checked_mul(u128::from(frames))?;

    if pixels == 0 {
      return None;
    }

    Some(Self { pixels, size })
  }
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
    fn case(size: u64, pixels: u64, expected: &str) {
      assert_eq!(
        DisplayBitsPerPixel::new(
          Dimensions {
            height: 1,
            width: pixels,
          },
          1,
          size,
        )
        .unwrap()
        .to_string(),
        expected,
      );
    }

    case(0, 480, "0");
    case(1, 100, "0.08");
    case(81, 8000, "0.081");
    case(1, 3, "2.667");
    case(3, 16, "1.5");
    case(1500, 480, "25");
  }

  #[test]
  fn overflow() {
    #[track_caller]
    fn case(height: u64, width: u64, frames: u64) {
      assert!(DisplayBitsPerPixel::new(Dimensions { height, width }, frames, 0).is_none());
    }

    case(0, 1, 1);
    case(1, 0, 1);
    case(1, 1, 0);
    case(u64::MAX, u64::MAX, u64::MAX);
  }
}
