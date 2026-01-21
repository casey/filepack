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

  pub(crate) fn prefix_and_suffix(&self) -> ([Fe32; 1], &[u8]) {
    match self {
      SignatureScheme::Filepack => ([Fe32::F], &[]),
      SignatureScheme::Pgp { hashed_area } => ([Fe32::P], hashed_area),
      SignatureScheme::Ssh => ([Fe32::S], &[]),
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
        hasher.update([4]);
        hasher.update([0]);
        hasher.update([22]);
        hasher.update([10]);
        hasher.update((hashed_area.len() as u16).to_be_bytes());

        // hashed area
        hasher.update(hashed_area);

        // trailer
        hasher.update([4]);
        hasher.update([0xff]);
        hasher.update((6 + hashed_area.len() as u32).to_be_bytes());

        hasher.finalize().to_vec().into()
      }
      Self::Ssh => {
        let mut buffer = b"SSHSIG".to_vec();

        let mut field = |value: &[u8]| {
          buffer.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
          buffer.extend_from_slice(value);
        };

        field(b"filepack");
        field(b"");
        field(b"sha512");
        field(&Sha512::digest(&message.bytes()));

        buffer.into()
      }
    }
  }
}
