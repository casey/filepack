use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Resolutions {
  max: Option<Dimensions>,
  min: Dimensions,
  shorthand: bool,
}

impl Resolutions {
  fn entry(&self, dimensions: Dimensions) -> String {
    if self.shorthand
      && let Some(shorthand) = dimensions.shorthand()
    {
      return shorthand.into();
    }

    dimensions.to_string()
  }

  pub(crate) fn new(
    dimensions: impl IntoIterator<Item = Dimensions>,
    shorthand: bool,
  ) -> Option<Self> {
    let mut dimensions = dimensions.into_iter().collect::<Vec<Dimensions>>();

    dimensions.sort_by_key(|dimensions| (dimensions.area(), dimensions.width));

    let min = *dimensions.first()?;
    let max = *dimensions.last()?;

    Some(Self {
      max: (max != min).then_some(max),
      min,
      shorthand,
    })
  }
}

impl Display for Resolutions {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.entry(self.min))?;

    if let Some(max) = self.max {
      write!(f, " – {}", self.entry(max))?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(dimensions: &[(u64, u64)], shorthand: bool, expected: &str) {
      assert_eq!(
        Resolutions::new(
          dimensions.iter().map(|(width, height)| Dimensions {
            height: *height,
            width: *width,
          }),
          shorthand,
        )
        .unwrap()
        .to_string(),
        expected,
      );
    }

    case(&[(2, 1)], false, "2×1");
    case(&[(2, 1), (2, 1)], false, "2×1");
    case(&[(4, 4), (2, 1)], false, "2×1 – 4×4");
    case(&[(4, 4), (2, 1), (3, 2)], false, "2×1 – 4×4");
    case(&[(4, 1), (2, 2), (1, 4)], false, "1×4 – 4×1");
    case(&[(1920, 1080)], false, "1920×1080");
    case(&[(1920, 1080)], true, "1080p");
    case(&[(3840, 2160), (2, 1), (1920, 1080)], true, "2×1 – 4K");
  }

  #[test]
  fn empty() {
    assert_eq!(Resolutions::new(Vec::new(), false), None);
  }
}
