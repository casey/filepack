use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum KeyIdentifierError {
  #[snafu(display("invalid public key name `{name}`"))]
  Name { name: String },
  #[snafu(transparent)]
  PublicKey { source: HexError },
}
