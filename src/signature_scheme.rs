use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SignatureScheme {
  Filepack,
}

impl From<SignatureScheme> for Fe32 {
  fn from(val: SignatureScheme) -> Self {
    match val {
      SignatureScheme::Filepack => Fe32::F,
    }
  }
}

impl TryFrom<Fe32> for SignatureScheme {
  type Error = SignatureError;

  fn try_from(scheme: Fe32) -> Result<Self, Self::Error> {
    match scheme {
      Fe32::F => Ok(Self::Filepack),
      _ => Err(signature_error::UnsupportedScheme { scheme }.build()),
    }
  }
}
