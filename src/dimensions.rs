use super::*;

#[derive(Clone, Copy, Debug, Default, Decode, Encode, PartialEq, Serialize)]
pub struct Dimensions {
  #[n(0)]
  pub(crate) height: u64,
  #[n(1)]
  pub(crate) width: u64,
}

impl Dimensions {
  pub(crate) fn shorthand(self) -> Option<&'static str> {
    match (self.width, self.height) {
      (1280, 720) => Some("720p"),
      (1920, 1080) => Some("1080p"),
      (2560, 1440) => Some("1440p"),
      (3840, 2160) => Some("4K"),
      (7680, 4320) => Some("8K"),
      _ => None,
    }
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
  fn encoding() {
    assert_cbor(
      Dimensions {
        height: 1,
        width: 2,
      },
      "a200010102",
    );
  }

  #[test]
  fn shorthand() {
    #[track_caller]
    fn case(width: u64, height: u64, expected: Option<&'static str>) {
      assert_eq!(Dimensions { height, width }.shorthand(), expected);
    }

    case(1280, 720, Some("720p"));
    case(1920, 1080, Some("1080p"));
    case(2560, 1440, Some("1440p"));
    case(3840, 2160, Some("4K"));
    case(7680, 4320, Some("8K"));
    case(720, 1280, None);
    case(2, 1, None);
  }
}
