use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SignatureScheme {
  Filepack,
  Pgp { hashed_area: Vec<u8> },
  Ssh,
}

impl SignatureScheme {
  pub(crate) fn new(scheme: Fe32, suffix: Vec<u8>) -> Result<Self, SignatureError> {
    match scheme {
      Fe32::F => {
        assert!(suffix.is_empty(), "todo: proper error");
        Ok(SignatureScheme::Filepack)
      }
      Fe32::P => Ok(SignatureScheme::Pgp {
        hashed_area: suffix,
      }),
      Fe32::S => {
        assert!(suffix.is_empty(), "todo: proper error");
        Ok(SignatureScheme::Ssh)
      }
      _ => return Err(signature_error::UnsupportedScheme { scheme }.build()),
    }
  }

  pub(crate) fn prefix(&self) -> Fe32 {
    match self {
      SignatureScheme::Filepack => Fe32::F,
      SignatureScheme::Pgp { .. } => Fe32::P,
      SignatureScheme::Ssh => Fe32::S,
    }
  }
}
