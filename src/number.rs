use super::*;

#[derive(Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Number(pub(crate) u64);

impl FromStr for Number {
  type Err = NumberError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(parse_number(s)?))
  }
}

impl PathExtractor for Number {
  fn parameters() -> Vec<reference::Parameter> {
    vec![reference::Parameter::Integer]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    assert_eq!("0".parse::<Number>().unwrap(), Number(0));
    assert_eq!("10".parse::<Number>().unwrap(), Number(10));
  }

  #[test]
  fn from_str_errors() {
    #[track_caller]
    fn case(s: &str, expected: &str) {
      assert_eq!(s.parse::<Number>().unwrap_err().to_string(), expected);
    }

    case("+1", "invalid number `+1`");
    case("01", "invalid number `01`");
    case("foo", "invalid number `foo`");
  }
}
