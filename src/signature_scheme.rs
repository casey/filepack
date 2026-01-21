use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn filepack_to_fe32() {
    assert_eq!(Fe32::from(SignatureScheme::Filepack), Fe32::F);
  }

  #[test]
  fn fe32_to_filepack() {
    assert_eq!(
      SignatureScheme::try_from(Fe32::F).unwrap(),
      SignatureScheme::Filepack,
    );
  }

  #[test]
  fn unsupported_scheme() {
    assert_eq!(
      SignatureScheme::try_from(Fe32::Q)
        .unwrap_err()
        .to_string(),
      "bech32m signature scheme `q` is not supported",
    );
  }
}
