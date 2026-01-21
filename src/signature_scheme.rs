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
        let hashed_area_len = hashed_area.len();

        let mut header = [0u8; 6];
        header[0] = 4;
        header[1] = 0;
        header[2] = 22;
        header[3] = 10;
        header[4..6].copy_from_slice(&(hashed_area_len as u16).to_be_bytes());

        let mut trailer = [0u8; 6];
        trailer[0] = 4;
        trailer[1] = 0xff;
        let len = (header.len() + hashed_area_len) as u32;
        trailer[2..6].copy_from_slice(&len.to_be_bytes());

        let mut hasher = Sha512::new();
        hasher.update(message.bytes());
        hasher.update(&header);
        hasher.update(hashed_area);
        hasher.update(&trailer);

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
