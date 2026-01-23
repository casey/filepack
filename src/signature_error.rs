use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SignatureError {
  #[snafu(transparent)]
  Bech32 { source: Bech32Error },
  #[snafu(display("{scheme} signature suffix length {length} exceeds maximum {maximum}",))]
  SuffixLength {
    length: usize,
    maximum: usize,
    scheme: SignatureSchemeType,
  },
  #[snafu(display("found unexpected {suffix} byte suffix on {scheme} signature"))]
  UnexpectedSuffix {
    suffix: usize,
    scheme: SignatureSchemeType,
  },
  #[snafu(display(
    "signature hash algorithm `{actual}` not supported with {scheme} signatures, \
    expected hash algorithm `{}`",
    scheme.hash_algorithm(),
  ))]
  UnsupportedHashAlgorithm {
    actual: Fe32,
    scheme: SignatureSchemeType,
  },
  #[snafu(display("signature scheme `{scheme}` not supported"))]
  UnsupportedScheme { scheme: Fe32 },
  #[snafu(display(
    "signature version `{actual}` not supported with {scheme} signatures, expected version `{}`",
    scheme.version(),
  ))]
  UnsupportedVersion {
    actual: Fe32,
    scheme: SignatureSchemeType,
  },
}
