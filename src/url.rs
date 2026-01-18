use super::*;

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Url(String);

impl FromStr for Url {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(s.into()))
  }
}
