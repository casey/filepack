use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum HashError {
  #[snafu(display("hashes must be lowercase hex: `{hash}`"))]
  Case { hash: String },
  #[snafu(transparent)]
  Hex { source: blake3::HexError },
}
