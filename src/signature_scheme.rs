use {
  super::*,
  sha2::{Digest, Sha512},
};

#[derive(Clone, Debug, EnumDiscriminants, PartialEq)]
#[strum_discriminants(name(SignatureSchemeType), vis(pub))]
pub(crate) enum SignatureScheme {
  Filepack,
  Pgp { hashed_area: Vec<u8> },
  Ssh,
}

impl SignatureSchemeType {
  fn new(scheme: Fe32, version: Fe32) -> Result<Self, SignatureError> {
    let scheme = match scheme {
      Fe32::F => Self::Filepack,
      Fe32::P => Self::Pgp,
      Fe32::S => Self::Ssh,
      _ => return Err(signature_error::UnsupportedScheme { scheme }.build()),
    };

    ensure! {
      version == scheme.version(),
      signature_error::UnsupportedVersion {
        scheme,
        actual: version,
      },
    }

    Ok(scheme)
  }

  fn prefix(self) -> [Fe32; 2] {
    let scheme = match self {
      Self::Filepack => Fe32::F,
      Self::Pgp => Fe32::P,
      Self::Ssh => Fe32::S,
    };

    [scheme, self.version()]
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
  pub(crate) fn new(scheme: Fe32, version: Fe32, suffix: Vec<u8>) -> Result<Self, SignatureError> {
    let scheme = SignatureSchemeType::new(scheme, version)?;

    match scheme {
      SignatureSchemeType::Filepack | SignatureSchemeType::Ssh => ensure!(
        suffix.is_empty(),
        signature_error::UnexpectedSuffix { scheme },
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
  ) -> Bech32mPayload<2, 64, Vec<u8>> {
    let prefix = self.discriminant().prefix();

    let suffix = match self {
      SignatureScheme::Filepack | SignatureScheme::Ssh => &[],
      SignatureScheme::Pgp { hashed_area } => hashed_area.as_slice(),
    };

    Bech32mPayload {
      prefix,
      data: signature.to_bytes(),
      suffix: suffix.into(),
    }
  }

  pub(crate) fn signed_data<'a>(&self, message: &'a SerializedMessage) -> Cow<'a, [u8]> {
    match self {
      Self::Filepack => message.bytes().into(),
      Self::Pgp { hashed_area } => {
        let mut hasher = Sha512::new();

        // message
        hasher.update(message.bytes());

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
        field(&Sha512::digest(message.bytes()));

        buffer.into()
      }
    }
  }
}
