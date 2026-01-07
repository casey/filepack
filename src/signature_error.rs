use super::*;

pub struct SignatureError(pub(crate) ed25519_dalek::SignatureError);

impl Display for SignatureError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl fmt::Debug for SignatureError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.0, f)
  }
}

impl std::error::Error for SignatureError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    self.0.source()?.source()
  }
}
