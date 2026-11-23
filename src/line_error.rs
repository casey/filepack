use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum LineError {
  #[snafu(display("line may not contain control character `{}`", character.escape_default()))]
  Control { character: char },
  #[snafu(display("line may not be empty"))]
  Empty,
  #[snafu(display("line may not contain leading or trailing whitespace"))]
  Whitespace,
}
