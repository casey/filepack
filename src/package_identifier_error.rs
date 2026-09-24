use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum PackageIdentifierError {
  #[snafu(transparent)]
  Fingerprint { source: HexError },
  #[snafu(transparent)]
  Number { source: NumberError },
}
