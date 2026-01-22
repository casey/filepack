use {
  super::*,
  sha2::{Digest, Sha512},
};

#[derive(Clone, Debug, EnumDiscriminants, PartialEq)]
#[strum_discriminants(name(SignatureSchemeType), derive(EnumIter), vis(pub))]
pub(crate) enum SignatureScheme {
  Filepack,
  Pgp { hashed_area: Vec<u8> },
  Ssh,
}

impl SignatureSchemeType {
  pub(crate) fn hash_algorithm(self) -> Fe32 {
    match self {
      Self::Filepack => Fe32::Q,
      Self::Ssh | Self::Pgp => Fe32::P,
    }
  }

  fn new(prefix: [Fe32; 3]) -> Result<Self, SignatureError> {
    let [scheme, version, hash] = prefix;

    let scheme = match scheme {
      Fe32::F => Self::Filepack,
      Fe32::P => Self::Pgp,
      Fe32::S => Self::Ssh,
      _ => return Err(signature_error::UnsupportedScheme { scheme }.build()),
    };

    ensure! {
      hash == scheme.hash_algorithm(),
      signature_error::UnsupportedHashAlgorithm {
        scheme,
        actual: hash,
      },
    }

    ensure! {
      version == scheme.version(),
      signature_error::UnsupportedVersion {
        scheme,
        actual: version,
      },
    }

    Ok(scheme)
  }

  fn prefix(self) -> [Fe32; 3] {
    let scheme = match self {
      Self::Filepack => Fe32::F,
      Self::Pgp => Fe32::P,
      Self::Ssh => Fe32::S,
    };

    [scheme, self.version(), self.hash_algorithm()]
  }

  pub(crate) fn version(self) -> Fe32 {
    match self {
      Self::Filepack | Self::Ssh => Fe32::_0,
      Self::Pgp => Fe32::_4,
    }
  }
}

impl Display for SignatureSchemeType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Filepack => write!(f, "filepack"),
      Self::Pgp => write!(f, "PGP"),
      Self::Ssh => write!(f, "SSH"),
    }
  }
}

impl SignatureScheme {
  pub(crate) fn new(prefix: [Fe32; 3], suffix: Vec<u8>) -> Result<Self, SignatureError> {
    let scheme = SignatureSchemeType::new(prefix)?;

    match scheme {
      SignatureSchemeType::Filepack | SignatureSchemeType::Ssh => ensure!(
        suffix.is_empty(),
        signature_error::UnexpectedSuffix {
          scheme,
          suffix: suffix.len(),
        },
      ),
      SignatureSchemeType::Pgp => {
        u16::try_from(suffix.len())
          .ok()
          .context(signature_error::SuffixLength {
            length: suffix.len(),
            maximum: usize::from(u16::MAX),
            scheme,
          })?;
      }
    }

    match scheme {
      SignatureSchemeType::Filepack => Ok(SignatureScheme::Filepack),
      SignatureSchemeType::Pgp => Ok(SignatureScheme::Pgp {
        hashed_area: suffix,
      }),
      SignatureSchemeType::Ssh => Ok(SignatureScheme::Ssh),
    }
  }

  pub(crate) fn payload(
    &self,
    signature: ed25519_dalek::Signature,
  ) -> Bech32mPayload<3, 64, &Vec<u8>> {
    static EMPTY: Vec<u8> = Vec::new();

    let prefix = self.discriminant().prefix();

    let suffix = match self {
      SignatureScheme::Filepack | SignatureScheme::Ssh => &EMPTY,
      SignatureScheme::Pgp { hashed_area } => hashed_area,
    };

    Bech32mPayload {
      body: signature.to_bytes(),
      prefix,
      suffix,
    }
  }

  pub(crate) fn signed_data<'a>(&self, message: &'a SerializedMessage) -> Cow<'a, [u8]> {
    match self {
      Self::Filepack => message.as_bytes().into(),
      Self::Pgp { hashed_area } => {
        let mut hasher = Sha512::new();

        // message
        hasher.update(message.as_bytes());

        // header
        hasher.update([4]); // version: 4
        hasher.update([0]); // signature type: binary signature
        hasher.update([22]); // public key algorithm: EdDSA
        hasher.update([10]); // hash algorithm: SHA-512
        hasher.update(u16::try_from(hashed_area.len()).unwrap().to_be_bytes());

        // hashed area
        hasher.update(hashed_area);

        // trailer
        hasher.update([4]); // version: 4
        hasher.update([0xff]); // marker byte
        hasher.update(u32::try_from(6 + hashed_area.len()).unwrap().to_be_bytes());

        hasher.finalize().to_vec().into()
      }
      Self::Ssh => {
        let mut buffer = b"SSHSIG".to_vec();

        let mut field = |value: &[u8]| {
          buffer.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
          buffer.extend_from_slice(value);
        };

        field(b"filepack"); // namespace
        field(b""); // reserved
        field(b"sha512"); // hash algorithm
        field(&Sha512::digest(message.as_bytes()));

        buffer.into()
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_algorithm_numeric_value() {
    assert_eq!(u8::from(SignatureSchemeType::Filepack.hash_algorithm()), 0);
    assert_eq!(u8::from(SignatureSchemeType::Pgp.hash_algorithm()), 1);
    assert_eq!(u8::from(SignatureSchemeType::Ssh.hash_algorithm()), 1);
  }

  #[test]
  fn round_trip() {
    for scheme in SignatureSchemeType::iter() {
      assert_eq!(SignatureSchemeType::new(scheme.prefix()).unwrap(), scheme);
    }
  }
}
