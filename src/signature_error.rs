use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SignatureError {
  #[snafu(display("invalid signature for key `{public_key}`"))]
  Invalid {
    public_key: PublicKey,
    source: ed25519_dalek::SignatureError,
  },
}
