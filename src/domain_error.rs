use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum DomainError {
  #[snafu(display("domain empty"))]
  Empty,
  #[snafu(display("domain contains empty label"))]
  EmptyLabel,
  #[snafu(display("invalid character {} in domain", c.escape_debug()))]
  InvalidCharacter { c: char },
  #[snafu(display("domain label length {len} exceeds 63 byte maximum"))]
  LabelLength { len: usize },
  #[snafu(display("domain label may not start with hyphen"))]
  LeadingHyphen,
  #[snafu(display("domain length {len} exceeds 253 byte maximum"))]
  Length { len: usize },
  #[snafu(display("domain must have at least two labels"))]
  TooFewLabels,
  #[snafu(display("domain label may not end with hyphen"))]
  TrailingHyphen,
}
