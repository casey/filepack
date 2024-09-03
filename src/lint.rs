use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Lint {
  Character { character: char },
  ComponentLength { length: usize },
  LeadingSpace,
  Name { name: String },
  TrailingPeriod,
  TrailingSpace,
}

impl Display for Lint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Lint::Character { character } => write!(f, "illegal character `{character}`"),
      Lint::ComponentLength { length } => write!(f, "length {length} path component"),
      Lint::LeadingSpace => write!(f, "leading space"),
      Lint::Name { name } => write!(f, "non-portable name `{name}`"),
      Lint::TrailingPeriod => write!(f, "trailing period"),
      Lint::TrailingSpace => write!(f, "trailing space"),
    }
  }
}

impl std::error::Error for Lint {}
