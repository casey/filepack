use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Bech32mError {
  #[snafu(display("failed to decode bech32m {ty}"))]
  Decode {
    ty: &'static str,
    source: CheckedHrpstringError,
  },
  #[snafu(display(
    "expected bech32m human-readable part `{expected}1...` but found `{actual}1...`",
  ))]
  Hrp {
    expected: crate::Hrp,
    actual: crate::Hrp,
  },
  #[snafu(display("expected {expected} bytes but found {actual}"))]
  Length { expected: usize, actual: usize },
  #[snafu(display("bech32m {ty} version `{version}` is unsupported"))]
  UnsupportedVersion {
    ty: &'static str,
    version: bech32::Fe32,
  },
  #[snafu(display("bech32m {ty} missing version character"))]
  VersionMissing { ty: &'static str },
}
