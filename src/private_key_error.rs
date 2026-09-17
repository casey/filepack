use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum PrivateKeyError {
  #[snafu(transparent)]
  Hex { source: HexError },
  #[snafu(display("private key derived public key does not match embedded public key"))]
  PublicKeyMismatch,
}
