use super::*;

#[derive(Clone, Copy, Debug, Default, Decode, Encode, PartialEq, Serialize)]
pub struct Dimensions {
  #[n(0)]
  pub(crate) height: u64,
  #[n(1)]
  pub(crate) width: u64,
}

impl Dimensions {
  pub(crate) fn css_aspect_ratio(self) -> String {
    format!("{} / {}", self.width.max(1), self.height.max(1))
  }
}

impl Display for Dimensions {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}×{}", self.width, self.height)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn css_aspect_ratio() {
    #[track_caller]
    fn case(width: u64, height: u64, expected: &str) {
      assert_eq!(Dimensions { height, width }.css_aspect_ratio(), expected);
    }

    case(2, 1, "2 / 1");
    case(1, 2, "1 / 2");
    case(0, 0, "1 / 1");
  }
}
