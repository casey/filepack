use super::*;

static NAME_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^[A-Za-z0-9][A-Za-z0-9._-]*$").unwrap());

#[derive(Clone, Debug, PartialEq)]
pub struct KeyName(String);

impl KeyName {
  pub(crate) fn master() -> Self {
    Self("master".into())
  }

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
