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
  #[snafu(display("expected bech32m {ty} to have {} but found {actual}", Count(*expected, "byte")))]
  Length {
    expected: usize,
    actual: usize,
    ty: &'static str,
  },
  #[snafu(display("bech32m {ty} has invalid padding"))]
  Padding {
    ty: &'static str,
    source: PaddingError,
  },
  #[snafu(display("bech32m {ty} missing {}", Count(*prefix, "prefix character")))]
  PrefixMissing { ty: &'static str, prefix: usize },
  #[snafu(display("bech32m {ty} version `{version}` is not supported"))]
  UnsupportedVersion {
    ty: &'static str,
    version: bech32::Fe32,
  },
  #[snafu(display("bech32m {ty} missing version character"))]
  VersionMissing { ty: &'static str },
}
