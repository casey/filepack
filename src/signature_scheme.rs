use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SignatureScheme {
  Filepack,
}

impl Into<Fe32> for SignatureScheme {
  fn into(self) -> Fe32 {
    match self {
      Self::Filepack => Fe32::F,
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
