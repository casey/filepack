use super::*;

pub struct DalekSignatureError(pub(crate) ed25519_dalek::SignatureError);

impl Display for DalekSignatureError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl fmt::Debug for DalekSignatureError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.0, f)
  }
}

impl std::error::Error for DalekSignatureError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    self.0.source()?.source()
  }
}
