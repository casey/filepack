use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum TimeError {
  #[snafu(display("day `{day}` out of range for month `{month}` of year `{year}`"))]
  Day { day: u8, month: u8, year: i64 },
  #[snafu(display("invalid time `{input}`"))]
  Invalid { input: String },
  #[snafu(display("month `{month}` out of range"))]
  Month { month: u8 },
  #[snafu(display("year `{year}` out of range"))]
  Year { year: i64 },
}
