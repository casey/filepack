use super::*;

#[derive(Debug, PartialEq, Snafu)]
pub enum TextError {
  #[snafu(display("text may not contain control character `{}`", character.escape_default()))]
  Control { character: char },
}
