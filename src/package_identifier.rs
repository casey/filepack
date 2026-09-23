use super::*;

#[derive(Clone, Copy, Debug, DeserializeFromStr, Eq, PartialEq)]
pub(crate) enum PackageIdentifier {
  Fingerprint(Fingerprint),
  Number(u64),
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
