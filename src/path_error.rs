use super::*;

#[derive(Debug, PartialEq, Snafu)]
pub enum PathError {
  #[snafu(display("paths may not contain non-normal component `{component}`"))]
  Component { component: String },
  #[snafu(display("paths may not contain empty components"))]
  ComponentEmpty,
  #[snafu(display("paths may not contain double slashes"))]
  DoubleSlash,
  #[snafu(display("paths may not be empty"))]
  Empty,
  #[snafu(display("paths may not begin with slash character"))]
  LeadingSlash,
  #[snafu(display("component exceeds 255 byte limit"))]
  Length,
  #[snafu(display("paths may not contain NUL character"))]
  Nul,
  #[snafu(display("paths may not contain separator character `{character}`"))]
  Separator { character: char },
  #[snafu(display("paths may not end with slash character"))]
  TrailingSlash,
  #[snafu(display("paths may not begin with Windows disk prefix `{letter}:`"))]
  WindowsDiskPrefix { letter: char },
}
