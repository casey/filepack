use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum BytesError {
  #[snafu(display("invalid hex digit `{}`", digit.escape_default()))]
  Digit { digit: char },
  #[snafu(display("odd number of hex digits: {len}"))]
  OddLength { len: usize },
}
