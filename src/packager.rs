use super::*;

#[derive(Debug, Snafu)]
pub enum PackagerError {
  #[snafu(display("packager may not contain `{character}`"))]
  Character { character: char },
}

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Packager(String);

impl FromStr for Packager {
  type Err = PackagerError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.contains('\n') {
      return Err(PackagerError::Character { character: '\n' });
    }

    Ok(Self(s.into()))
  }
}
