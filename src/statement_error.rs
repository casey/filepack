use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum StatementError {
  #[snafu(display(
    "signature fingerprint `{signature}` does not match package fingerprint `{package}`"
  ))]
  SignatureFingerprintMismatch {
    package: Fingerprint,
    signature: Fingerprint,
  },
  #[snafu(transparent)]
  SignatureInvalid { source: SignatureError },
}
