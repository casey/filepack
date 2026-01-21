use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SignatureError {
  #[snafu(transparent)]
  Bech32m { source: Bech32mError },
  #[snafu(display("signature scheme `{scheme}` does not support suffix data"))]
  SuffixNotAllowed { scheme: &'static str },
  #[snafu(display(
    "signature scheme `{scheme}` suffix length {length} exceeds maximum {}",
    u16::MAX,
  ))]
  SuffixTooLarge { length: usize, scheme: &'static str },
  #[snafu(display("bech32m signature scheme `{scheme}` is not supported"))]
  UnsupportedScheme { scheme: Fe32 },
}
