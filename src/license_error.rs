use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum LicenseError {
  #[snafu(display("SPDX license identifier `{license}` is deprecated"))]
  Deprecated { license: String },
  #[snafu(display(
    "unknown license `{license}`: expected `permission`, `public-domain`, `reserved`, `unknown`, \
     or an SPDX license identifier"
  ))]
  Unknown { license: String },
}
