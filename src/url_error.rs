use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum UrlError {
  #[snafu(transparent)]
  Parse { source: url::ParseError },
  #[snafu(display("URL scheme `{scheme}` not allowed, must be `http` or `https`"))]
  Scheme { scheme: String },
}
