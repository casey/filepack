use super::*;

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Packager(String);

impl FromStr for Packager {
  type Err = std::convert::Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(s.into()))
  }
}
