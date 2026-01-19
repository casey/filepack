use {super::*, bech32::primitives::decode::CheckedHrpstringError};

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Bech32mError {
  #[snafu(display("failed to decode bech32m {ty}"))]
  Decode {
    ty: &'static str,
    source: CheckedHrpstringError,
  },
  #[snafu(display(
    "expected bech32m human-readable prefix `{expected}1...` but found `{actual}1...`",
  ))]
  Hrp {
    expected: crate::Hrp,
    actual: crate::Hrp,
  },
  #[snafu(display("expected {expected} bytes but found {actual}"))]
  Length { expected: usize, actual: usize },
}
