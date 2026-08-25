use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum XmpError {
  #[snafu(display("missing element `{name}`"))]
  MissingElement { name: &'static str },
  #[snafu(display("failed to parse XML"))]
  Parse { source: roxmltree::Error },
  #[snafu(display("multiple `dc:title` values"))]
  TitleMultiple,
  #[snafu(display("unexpected element `{name}`"))]
  UnexpectedElement { name: String },
  #[snafu(display("invalid UTF-8"))]
  Utf8 { source: str::Utf8Error },
}
