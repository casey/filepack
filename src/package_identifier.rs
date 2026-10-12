use super::*;

#[derive(Clone, Copy, Debug, DeserializeFromStr, PartialEq)]
pub enum PackageIdentifier {
  Fingerprint(Fingerprint),
  Number(u64),
  Revision(Revision),
}

#[cfg(test)]
impl Default for PackageIdentifier {
  fn default() -> Self {
    Self::Number(0)
  }
}

impl Display for PackageIdentifier {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Fingerprint(fingerprint) => write!(f, "{fingerprint}"),
      Self::Number(number) => write!(f, "{number}"),
      Self::Revision(revision) => write!(f, "{revision}"),
    }
  }
}

impl FromStr for PackageIdentifier {
  type Err = PackageIdentifierError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.starts_with(|c: char| c.is_ascii_digit()) {
      Ok(Self::Number(parse_number(s)?))
    } else if s.starts_with(Tag::Package.prefix()) {
      Ok(Self::Fingerprint(s.parse()?))
    } else if s.starts_with(Tag::Revision.prefix()) {
      Ok(Self::Revision(s.parse()?))
    } else {
      Err(PackageIdentifierError::Unrecognized {
        identifier: s.into(),
      })
    }
  }
}

impl PathExtractor for PackageIdentifier {
  fn parameters() -> Vec<reference::Parameter> {
    vec![reference::Parameter::PackageIdentifier]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(PackageIdentifier::Number(1).to_string(), "1");
    assert_eq!(
      PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()).to_string(),
      Fingerprint::EXAMPLE,
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
      Fingerprint::EXAMPLE,
      PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()),
    );
    case(
      Revision::EXAMPLE,
      PackageIdentifier::Revision(Revision::EXAMPLE.parse().unwrap()),
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
    case("foo", "unrecognized package identifier `foo`");
    case(
      "package1ZZ",
      "package fingerprint contains invalid hex digit `Z`",
    );
    case("revision1ZZ", "revision contains invalid hex digit `Z`");
  }
}
