use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ImageFormat {
  pub(crate) dimensions: Dimensions,
  pub(crate) ty: ImageType,
}

impl Display for ImageFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} · {}", self.ty, self.dimensions)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(ty: ImageType, width: u64, height: u64, expected: &str) {
      assert_eq!(
        ImageFormat {
          dimensions: Dimensions { height, width },
          ty,
        }
        .to_string(),
        expected,
      );
    }

    case(ImageType::Png, 2, 1, "PNG · 2×1");
    case(ImageType::Jpeg, 1920, 1080, "JPEG · 1920×1080");
  }
}
