use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SignatureError {
  #[snafu(transparent)]
  Bech32m { source: Bech32mError },
  #[snafu(display("bech32m signature missing scheme character"))]
  SchemeMissing,
  #[snafu(display("bech32m signature scheme `{scheme}` is not supported"))]
  UnsupportedScheme { scheme: Fe32 },
}
