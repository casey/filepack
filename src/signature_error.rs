use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SignatureError {
  #[snafu(transparent)]
  Bech32 { source: Bech32Error },
  #[snafu(display("signature public key invalid")]
  PublicKey { source: PublicKeyError },
}
