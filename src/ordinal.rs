use super::*;

#[derive(Clone, Copy, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Ordinal(pub(crate) usize);

impl Display for Ordinal {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0 + 1)
  }
}

impl FromStr for Ordinal {
  type Err = OrdinalError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.parse::<usize>()?
      .checked_sub(1)
      .map(Self)
      .context(ordinal_error::Zero)
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
    assert_matches!("0".parse::<Ordinal>(), Err(OrdinalError::Zero));
    assert_matches!("foo".parse::<Ordinal>(), Err(OrdinalError::Int { .. }));
  }
}
