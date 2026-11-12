use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum KeyError {
  #[snafu(transparent)]
  Filesystem { source: FilesystemError },
  #[snafu(display("invalid private key `{path}`"))]
  PrivateKeyLoad {
    path: DisplayPath,
    source: PrivateKeyError,
  },
  #[snafu(display("private key not found: `{path}`"))]
  PrivateKeyNotFound { path: DisplayPath },
  #[snafu(display("invalid public key: `{path}`"))]
  PublicKeyLoad { path: DisplayPath, source: HexError },
  #[snafu(display("public key not found: `{path}`"))]
  PublicKeyNotFound { path: DisplayPath },
}
