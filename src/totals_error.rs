use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum TotalsError {
  #[snafu(display("totals mismatch, found {actual} but expected {expected}"))]
  Mismatch { actual: Totals, expected: Totals },
  #[snafu(display("totals overflowed"))]
  Overflow,
}
