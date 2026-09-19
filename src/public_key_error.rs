use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum PublicKeyError {
  #[snafu(display("invalid public key: `{key}`"))]
  Invalid {
    key: InvalidPublicKey,
    source: ed25519_dalek::SignatureError,
  },
  #[snafu(display("weak public key: `{key}`"))]
  Weak { key: InvalidPublicKey },
}
