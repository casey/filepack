use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum UrlError {
  #[snafu(display("URL may not contain control character `{}`", character.escape_default()))]
  Control { character: char },
  #[snafu(display("URL may not be empty"))]
  Empty,
  #[snafu(display("URL with scheme `{scheme}` must have an authority"))]
  MissingAuthority { scheme: String },
  #[snafu(display("URL is missing a scheme"))]
  MissingScheme,
  #[snafu(display("invalid URL scheme `{scheme}`"))]
  Scheme { scheme: String },
  #[snafu(display("URL scheme `{scheme}` not allowed, must be `http` or `https`"))]
  UnknownScheme { scheme: String },
  #[snafu(display("URL may not contain whitespace"))]
  Whitespace,
}
