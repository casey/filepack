use super::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Domain(String);

impl Domain {
  pub(crate) fn is_equivalent(&self, other: impl AsRef<str>) -> bool {
    self.0.eq_ignore_ascii_case(other.as_ref())
  }

  fn validate_label(label: &str) -> Result<(), DomainError> {
    if label.is_empty() {
      return Err(DomainError::EmptyLabel);
    }

    if label.len() > 63 {
      return Err(DomainError::LabelLength { len: label.len() });
    }

    for c in label.chars() {
      if !(c.is_ascii_alphanumeric() || c == '-') {
        return Err(DomainError::InvalidCharacter { c });
      }
    }

    if label.starts_with('-') {
      return Err(DomainError::LeadingHyphen);
    }

    if label.ends_with('-') {
      return Err(DomainError::TrailingHyphen);
    }

    Ok(())
  }
}

impl AsRef<str> for Domain {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
  }
}

impl Display for Domain {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl FromStr for Domain {
  type Err = DomainError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.is_empty() {
      return Err(DomainError::Empty);
    }

    if s.len() > 253 {
      return Err(DomainError::Length { len: s.len() });
    }

    let mut labels = 0;
    for label in s.split('.') {
      Self::validate_label(label)?;
      labels += 1;
    }

    if labels < 2 {
      return Err(DomainError::TooFewLabels);
    }

    Ok(Domain(s.to_owned()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    #[track_caller]
    fn case(s: &str) {
      assert!(s.parse::<Domain>().is_ok());
    }

    case("example.com");
    case("a.b");
    case("sub.domain.example.com");
    case("foo-bar.example.com");
    case("123.example.com");
    case("xn--mnchen-3ya.de");
  }

  #[test]
  fn display() {
    let d = "example.com".parse::<Domain>().unwrap();
    assert_eq!(d.to_string(), "example.com");
  }

  #[test]
  fn parse_error() {
    #[track_caller]
    fn case(s: &str, expected: DomainError) {
      assert_eq!(s.parse::<Domain>().unwrap_err(), expected)
    }

    case("localhost", DomainError::TooFewLabels);
    case("", DomainError::Empty);
    case("-example.com", DomainError::LeadingHyphen);
    case("example-.com", DomainError::TrailingHyphen);
    case("example..com", DomainError::EmptyLabel);
    case(".example.com", DomainError::EmptyLabel);
    case("example.com.", DomainError::EmptyLabel);
    case("exa_mple.com", DomainError::InvalidCharacter { c: '_' });
    case("münchen.de", DomainError::InvalidCharacter { c: 'ü' });
    case(
      &format!("{}.com", "a".repeat(64)),
      DomainError::LabelLength { len: 64 },
    );
    case(
      &["a".repeat(63).as_str(); 4].join("."),
      DomainError::Length { len: 255 },
    );
  }
}
