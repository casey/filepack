use super::*;

#[derive(Clone, Debug, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub(crate) struct Url(String);

impl Url {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for Url {
  type Err = ::url::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.parse::<::url::Url>()?;
    Ok(Self(s.into()))
  }
}

impl Display for Url {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{self}")
  }
}
