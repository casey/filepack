use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Bech32mError {
  #[snafu(display(
    "expected bech32m {ty} to have {} but found {actual}",
    Count(*expected, "body byte"),
  ))]
  BodyLength {
    expected: usize,
    actual: usize,
    ty: Bech32mType,
  },
  #[snafu(display("failed to decode bech32m {ty}"))]
  Decode {
    ty: Bech32mType,
    source: CheckedHrpstringError,
  },
  #[snafu(display(
    "expected bech32m human-readable part `{}1...` but found `{actual}1...`",
    ty.hrp(),
  ))]
  Hrp { ty: Bech32mType, actual: crate::Hrp },
  #[snafu(display("bech32m {ty} has invalid padding"))]
  Padding {
    ty: Bech32mType,
    source: PaddingError,
  },
  #[snafu(display(
    "expected bech32m {ty} to have {} but found {actual}",
    Count(*expected, "prefix character"),
  ))]
  PrefixLength {
    expected: usize,
    actual: usize,
    ty: Bech32mType,
  },
  #[snafu(display(
    "expected bech32m {ty} to have {} but found {actual}",
    Count(*expected, "suffix byte"),
  ))]
  SuffixLength {
    expected: usize,
    actual: usize,
    ty: Bech32mType,
  },
  #[snafu(display("bech32m {ty} version `{version}` is not supported"))]
  UnsupportedVersion {
    ty: Bech32mType,
    version: bech32::Fe32,
  },
  #[snafu(display("bech32m {ty} missing version character"))]
  VersionMissing { ty: Bech32mType },
}
