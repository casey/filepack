use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum PrivateKeyError {
  #[snafu(transparent)]
  Bech32 { source: Bech32Error },
  #[snafu(display("private key public key mismatch"))]
  Mismatch,
}
