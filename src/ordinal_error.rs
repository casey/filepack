use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum OrdinalError {
  #[snafu(transparent)]
  Int { source: ParseIntError },
  #[snafu(display("ordinal may not be zero"))]
  Zero,
}
