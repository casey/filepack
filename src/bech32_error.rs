use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Bech32Error {
  #[snafu(display("failed to decode bech32 {ty}"))]
  Decode {
    ty: Bech32Type,
    source: CheckedHrpstringError,
  },
  #[snafu(display(
    "expected bech32 human-readable part `{}1...` but found `{actual}1...`",
    ty.hrp(),
  ))]
  Hrp { ty: Bech32Type, actual: crate::Hrp },
  #[snafu(display("bech32 {ty} overlong by {}", Count(*excess, "character")))]
  Overlong { excess: usize, ty: Bech32Type },
  #[snafu(display("bech32 {ty} has nonzero padding"))]
  Padding { ty: Bech32Type },
  #[snafu(display("bech32 {ty} truncated"))]
  Truncated { ty: Bech32Type },
  #[snafu(display("bech32 {ty} version `{version}` not supported"))]
  UnsupportedVersion {
    ty: Bech32Type,
    version: bech32::Fe32,
  },
}
