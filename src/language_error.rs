use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum LanguageError {
  #[snafu(display("unknown language code `{code}`"))]
  Code { code: String },
}
