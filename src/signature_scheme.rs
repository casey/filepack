use {
  super::*,
  sha2::{Digest, Sha512},
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SignatureScheme {
  Filepack,
  Pgp { hashed_area: Vec<u8> },
  Ssh,
}

impl SignatureScheme {
  pub(crate) fn new(scheme: Fe32, version: Fe32, suffix: Vec<u8>) -> Result<Self, SignatureError> {
    match scheme {
      Fe32::F => {
        ensure!(
          version == Fe32::_0,
          signature_error::UnsupportedSchemeVersion {
            scheme: "filepack",
            version,
            expected: Fe32::_0,
          },
        );
        ensure!(
          suffix.is_empty(),
          signature_error::UnexpectedSuffix { scheme: "filepack" },
        );
        Ok(SignatureScheme::Filepack)
      }
      Fe32::P => {
        ensure!(
          version == Fe32::_4,
          signature_error::UnsupportedSchemeVersion {
            scheme: "pgp",
            version,
            expected: Fe32::_4,
          },
        );
        u16::try_from(suffix.len())
          .ok()
          .context(signature_error::SuffixLength {
            length: suffix.len(),
            maximum: usize::from(u16::MAX),
            scheme: "pgp",
          })?;
        Ok(SignatureScheme::Pgp {
          hashed_area: suffix,
        })
      }
      Fe32::S => {
        ensure!(
          version == Fe32::_0,
          signature_error::UnsupportedSchemeVersion {
            scheme: "ssh",
            version,
            expected: Fe32::_0,
          },
        );
        ensure!(
          suffix.is_empty(),
          signature_error::UnexpectedSuffix { scheme: "ssh" },
        );
        Ok(SignatureScheme::Ssh)
      }
      _ => Err(signature_error::UnsupportedScheme { scheme }.build()),
    }
  }

  pub(crate) fn prefix_and_suffix(&self) -> ([Fe32; 2], &[u8]) {
    match self {
      SignatureScheme::Filepack => ([Fe32::F, Fe32::_0], &[]),
      SignatureScheme::Pgp { hashed_area } => ([Fe32::P, Fe32::_4], hashed_area),
      SignatureScheme::Ssh => ([Fe32::S, Fe32::_0], &[]),
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
