use super::*;

#[derive(Debug, PartialEq, Snafu)]
pub enum ComponentError {
  #[snafu(display("component may not be `{component}`"))]
  Normal { component: &'static str },
  #[snafu(display("component may not be empty"))]
  Empty,
  #[snafu(display("component exceeds 255 byte limit"))]
  Length,
  #[snafu(display("component may not contain NUL character"))]
  Nul,
  #[snafu(display("component may not contain path separator `{character}`"))]
  Separator { character: char },
  #[snafu(display("component may not begin with Windows drive letter `{letter}:`"))]
  WindowsDriveLetter { letter: char },
}
