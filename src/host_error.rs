use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum HostError {
  #[snafu(display("invalid bracketed IPv6 address `{host}`"))]
  BracketedIpv6 { host: String },
  #[snafu(display("bracketed IPv6 address `{host}` missing closing `]`"))]
  BracketedIpv6Unterminated { host: String },
  #[snafu(display("domain `{domain}` contains invalid character `{character}`"))]
  DomainCharacter { character: char, domain: String },
  #[snafu(display("domain is empty"))]
  DomainEmpty,
  #[snafu(display("domain `{domain}` contains empty label"))]
  DomainLabelEmpty { domain: String },
  #[snafu(display("domain `{domain}` label `{label}` begins or ends with hyphen"))]
  DomainLabelHyphen { domain: String, label: String },
  #[snafu(display("domain `{domain}` label `{label}` is longer than 63 characters"))]
  DomainLabelLength { domain: String, label: String },
  #[snafu(display("domain `{domain}` is longer than 253 characters"))]
  DomainLength { domain: String },
  #[snafu(display("domain `{domain}` ends with numeric label"))]
  DomainNumeric { domain: String },
}
