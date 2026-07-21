use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Resolutions {
  dimensions: Vec<Dimensions>,
  shorthand: bool,
}

impl Resolutions {
  pub(crate) fn entries(&self) -> Vec<String> {
    match self.dimensions.as_slice() {
      [first, .., last] if self.dimensions.len() > 2 => {
        vec![format!("{} – {}", self.entry(*first), self.entry(*last))]
      }
      dimensions => dimensions
        .iter()
        .map(|dimensions| self.entry(*dimensions))
        .collect(),
    }
  }

  fn entry(&self, dimensions: Dimensions) -> String {
    if self.shorthand
      && let Some(shorthand) = dimensions.shorthand()
    {
      return shorthand.into();
    }

    dimensions.to_string()
  }

  pub(crate) fn new(dimensions: impl IntoIterator<Item = Dimensions>, shorthand: bool) -> Self {
    let mut distinct = Vec::<Dimensions>::new();

    for dimensions in dimensions {
      if !distinct.contains(&dimensions) {
        distinct.push(dimensions);
      }
    }

    distinct.sort_by_key(|dimensions| (dimensions.area(), dimensions.width));

    Self {
      dimensions: distinct,
      shorthand,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn entries() {
    #[track_caller]
    fn case(dimensions: &[(u64, u64)], shorthand: bool, expected: &[&str]) {
      assert_eq!(
        Resolutions::new(
          dimensions.iter().map(|(width, height)| Dimensions {
            height: *height,
            width: *width,
          }),
          shorthand,
        )
        .entries(),
        expected,
      );
    }

    case(&[], false, &[]);
    case(&[(2, 1)], false, &["2×1"]);
    case(&[(2, 1), (2, 1)], false, &["2×1"]);
    case(&[(4, 4), (2, 1)], false, &["2×1", "4×4"]);
    case(&[(4, 4), (2, 1), (3, 2)], false, &["2×1 – 4×4"]);
    case(&[(4, 1), (2, 2), (1, 4)], false, &["1×4 – 4×1"]);
    case(&[(1920, 1080)], false, &["1920×1080"]);
    case(&[(1920, 1080)], true, &["1080p"]);
    case(&[(1920, 1080), (2, 1)], true, &["2×1", "1080p"]);
    case(&[(3840, 2160), (2, 1), (1920, 1080)], true, &["2×1 – 4K"]);
  }
}
