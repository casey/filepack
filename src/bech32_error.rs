use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Bech32Error {
  #[snafu(display("failed to decode bech32m {ty}"))]
  Decode {
    ty: Bech32Type,
    source: CheckedHrpstringError,
  },
  #[snafu(display(
    "expected bech32m human-readable part `{}1...` but found `{actual}1...`",
    ty.hrp(),
  ))]
  Hrp { ty: Bech32Type, actual: crate::Hrp },
  #[snafu(display("bech32m {ty} overlong by {}", Count(*excess, "characters")))]
  Overlong { excess: usize, ty: Bech32Type },
  #[snafu(display("bech32m {ty} has invalid padding"))]
  Padding {
    ty: Bech32Type,
    source: PaddingError,
  },
  #[snafu(display("bech32m {ty} truncated"))]
  Truncated { ty: Bech32Type },
  #[snafu(display("bech32m {ty} version `{version}` is not supported"))]
  UnsupportedVersion {
    ty: Bech32Type,
    version: bech32::Fe32,
  },
}
