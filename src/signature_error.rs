use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SignatureError {
  #[snafu(transparent)]
  Bech32 { source: Bech32Error },
  #[snafu(transparent)]
  PublicKey { source: PublicKeyError },
}
