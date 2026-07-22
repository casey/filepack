use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum TimeError {
  #[snafu(display("epoch day `{days}` out of range"))]
  Days { days: i32 },
  #[snafu(display("invalid time `{input}`"))]
  Invalid { input: String },
}
