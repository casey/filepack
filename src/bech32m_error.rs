use {super::*, bech32::primitives::decode::CheckedHrpstringError};

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Bech32mError {
  #[snafu(display("failed to decode bech32m"))]
  Decode { source: CheckedHrpstringError },
  #[snafu(display("unexpected hrp"))]
  Hrp {
    expected: crate::Hrp,
    actual: crate::Hrp,
  },
  #[snafu(display("unexpected length"))]
  Length { expected: usize, actual: usize },
}
