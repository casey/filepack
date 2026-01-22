use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SignatureError {
  #[snafu(transparent)]
  Bech32m { source: Bech32mError },
  #[snafu(display("signature scheme `{scheme}` suffix length {length} exceeds maximum {maximum}",))]
  SuffixLength {
    length: usize,
    maximum: usize,
    scheme: SignatureSchemeType,
  },
  #[snafu(display("found unexpected suffix for signature scheme `{scheme}`"))]
  UnexpectedSuffix { scheme: SignatureSchemeType },
  #[snafu(display("signature scheme `{scheme}` is not supported"))]
  UnsupportedScheme { scheme: Fe32 },
  #[snafu(display(
    "signature scheme `{scheme}` version `{actual}` is not supported, expected `{}`",
    scheme.version(),
  ))]
  UnsupportedVersion {
    actual: Fe32,
    scheme: SignatureSchemeType,
  },
}
