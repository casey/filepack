use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct KeyName(Cow<'static, str>);

impl KeyName {
  pub(crate) const DEFAULT: Self = KeyName(Cow::Borrowed("master"));

  fn filename(&self, key_type: KeyType) -> String {
    format!("{}.{}", self.0, key_type.extension())
  }

  pub(crate) fn private_key_filename(&self) -> String {
    self.filename(KeyType::Private)
  }

  pub(crate) fn public_key_filename(&self) -> String {
    self.filename(KeyType::Public)
  }
}

impl FromStr for KeyName {
  type Err = PublicKeyError;

  fn from_str(name: &str) -> Result<Self, Self::Err> {
    if re::PUBLIC_KEY.is_match(name) || !re::KEY_NAME.is_match(name) || name.len() > 128 {
      return Err(public_key_error::NameError { name }.build());
    }

    Ok(Self(Cow::Owned(name.into())))
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
    valid(&"a".repeat(128));

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
    invalid(&"a".repeat(129));
    invalid(&"a".repeat(64));
  }
}
