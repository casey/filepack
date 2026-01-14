use super::*;

static NAME_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

#[derive(Clone, Debug, PartialEq)]
pub struct KeyName(String);

impl KeyName {
  pub(crate) fn private_key_filename(&self) -> String {
    format!("{}.private", self.0)
  }

  pub(crate) fn public_key_filename(&self) -> String {
    format!("{}.public", self.0)
  }
}

impl FromStr for KeyName {
  type Err = PublicKeyError;

  fn from_str(name: &str) -> Result<Self, Self::Err> {
    if !NAME_RE.is_match(name) {
      return Err(public_key_error::NameError { name }.build());
    }

    Ok(Self(name.into()))
  }
}

impl Display for KeyName {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validity() {
    #[track_caller]
    fn valid(s: &str) {
      assert!(s.parse::<KeyName>().is_ok());
    }

    #[track_caller]
    fn invalid(s: &str) {
      assert!(s.parse::<KeyName>().is_err());
    }

    valid("0");
    valid("0-0");
    valid("foo");
    valid("foo-bar");
    valid("foo-bar-baz");

    invalid("");
    invalid("-");
    invalid("--");
    invalid(".");
    invalid("..");
    invalid("/");
    invalid("FOO");
    invalid("\\");
    invalid("_");
    invalid("foo--bar");
    invalid("foo--bar");
    invalid("foo-bar-");
    invalid("foo.");
    invalid("foo.bar");
    invalid("foo_bar");
  }
}
