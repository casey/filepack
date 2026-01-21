use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum PublicKeyError {
  #[snafu(transparent)]
  Bech32m { source: Bech32mError },
  #[snafu(display("invalid public key: `{key}`"))]
  Invalid {
    key: String,
    source: DalekSignatureError,
  },
  #[snafu(display("invalid public key name `{name}`"))]
  Name { name: String },
  #[snafu(display("weak public key: `{key}`"))]
  Weak { key: String },
}
