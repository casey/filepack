use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum TextError {
  #[snafu(display("text may not contain control character `{}`", character.escape_default()))]
  Control { character: char },
  #[snafu(display("text may not be empty"))]
  Empty,
  #[snafu(display("text may not contain leading or trailing whitespace"))]
  Whitespace,
}
