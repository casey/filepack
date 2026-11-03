use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum HostError {
  #[snafu(display("invalid bracketed IPv6 address `{host}`"))]
  BracketedIpv6 { host: String },
  #[snafu(display("domain `{host}` contains invalid character `{character}`"))]
  DomainCharacter { character: char, host: String },
  #[snafu(display("domain is empty"))]
  DomainEmpty,
  #[snafu(display("domain `{host}` contains empty label"))]
  DomainLabelEmpty { host: String },
  #[snafu(display("domain `{host}` label `{label}` begins or ends with hyphen"))]
  DomainLabelHyphen { host: String, label: String },
  #[snafu(display("domain `{host}` label `{label}` is longer than 63 characters"))]
  DomainLabelLength { host: String, label: String },
  #[snafu(display("domain `{host}` is longer than 253 characters"))]
  DomainLength { host: String },
  #[snafu(display("domain `{host}` ends with numeric label"))]
  DomainNumeric { host: String },
}
