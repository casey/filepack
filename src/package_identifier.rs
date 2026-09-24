use super::*;

#[derive(Clone, Copy, Debug, DeserializeFromStr, Eq, PartialEq)]
pub enum PackageIdentifier {
  Fingerprint(Fingerprint),
  Number(u64),
}

impl Display for PackageIdentifier {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Fingerprint(fingerprint) => write!(f, "{fingerprint}"),
      Self::Number(number) => write!(f, "{number}"),
    }
  }
}

impl FromStr for PackageIdentifier {
  type Err = PackageIdentifierError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.starts_with(|c: char| c.is_ascii_digit()) {
      Ok(Self::Number(parse_number(s)?))
    } else {
      Ok(Self::Fingerprint(s.parse()?))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(PackageIdentifier::Number(1).to_string(), "1");
    assert_eq!(
      PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()).to_string(),
      test::FINGERPRINT,
    );
  }

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: PackageIdentifier) {
      assert_eq!(s.parse::<PackageIdentifier>().unwrap(), expected);
    }

    case("0", PackageIdentifier::Number(0));
    case("1", PackageIdentifier::Number(1));
    case("10", PackageIdentifier::Number(10));
    case(
      test::FINGERPRINT,
      PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
    );
  }

  #[test]
  fn from_str_errors() {
    #[track_caller]
    fn case(s: &str, expected: &str) {
      assert_eq!(
        s.parse::<PackageIdentifier>().unwrap_err().to_string(),
        expected,
      );
    }

    case("01", "invalid number `01`");
    case("1a", "invalid number `1a`");
    case("foo", "package fingerprint missing tag `package1…`");
    case(
      "package1ZZ",
      "package fingerprint contains invalid hex digit `Z`",
    );
  }
}
