use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum LicenseError {
  #[snafu(display("SPDX license identifier `{license}` is deprecated"))]
  Deprecated { license: String },
  #[snafu(display("unknown SPDX license identifier `{license}`"))]
  Unknown { license: String },
}
