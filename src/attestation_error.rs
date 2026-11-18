use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum AttestationError {
  #[snafu(display("signature revision `{signature}` does not match package revision `{package}`"))]
  Revision {
    package: RevisionHash,
    signature: RevisionHash,
  },
  #[snafu(transparent)]
  Signature { source: SignatureError },
}
