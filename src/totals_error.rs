use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum TotalsError {
  #[snafu(display("totals overflowed 64-bit integer"))]
  Overflow,
  #[snafu(display("totals mismatch, found {actual} but expected {expected}"))]
  Mismatch { actual: Totals, expected: Totals },
}
