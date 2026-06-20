use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum DomainError {
  #[snafu(display("domain empty"))]
  Empty,
  #[snafu(display("domain of {len} bytes exceeds maximum of 253"))]
  Length { len: usize },
  #[snafu(display("domain must have at least two labels"))]
  TooFewLabels,
  #[snafu(display("domain contains empty label"))]
  EmptyLabel,
  #[snafu(display("label of {len} bytes exceeds maximum of 63"))]
  LabelLength { len: usize },
  #[snafu(display("invalid character {} in domain", c.escape_debug()))]
  Character { c: char },
  #[snafu(display("label may not start with a hyphen"))]
  LeadingHyphen,
  #[snafu(display("label may not end with a hyphen"))]
  TrailingHyphen,
}
