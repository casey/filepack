use super::*;

#[derive(Clone, Copy, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Ordinal(pub(crate) usize);

impl Display for Ordinal {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0 + 1)
  }
}

impl FromStr for Ordinal {
  type Err = ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(s.parse::<NonZeroUsize>()?.get() - 1))
  }
}

impl From<usize> for Ordinal {
  fn from(i: usize) -> Self {
    Self(i)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(Ordinal(0).to_string(), "1");
    assert_eq!(Ordinal(9).to_string(), "10");
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: Ordinal) {
      assert_eq!(s.parse::<Ordinal>().unwrap(), expected);
    }

    case("1", Ordinal(0));
    case("5", Ordinal(4));
  }

  #[test]
  fn from_str_errors() {
    #[track_caller]
    fn case(s: &str, expected: &str) {
      assert_eq!(s.parse::<Ordinal>().unwrap_err().to_string(), expected);
    }

    case("0", "number would be zero for non-zero type");
    case("foo", "invalid digit found in string");
  }
}
