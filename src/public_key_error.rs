use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)), visibility(pub(crate)))]
pub enum PublicKeyError {
  #[snafu(display("public keys must be lowercase hex: `{key}`"))]
  Case { key: String },
  #[snafu(display("invalid public key hex: `{key}`"))]
  Hex {
    key: String,
    source: hex::FromHexError,
  },
  #[snafu(display("invalid public key: `{key}`"))]
  Invalid { key: String, source: SignatureError },
  #[snafu(display("invalid public key byte length {length}: `{key}`"))]
  Length {
    key: String,
    length: usize,
    source: TryFromSliceError,
  },
  #[snafu(display("invalid public key name `{name}`"))]
  Name { name: String },
  #[snafu(display("weak public key: `{key}`"))]
  Weak { key: String },
}
