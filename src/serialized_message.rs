use sha2::{Digest, Sha512};

pub struct SerializedMessage(pub(crate) Vec<u8>);

impl SerializedMessage {
  pub(crate) fn ssh_signed_data(&self) -> Vec<u8> {
    let mut buffer = b"SSHSIG".to_vec();

    let mut field = |value: &[u8]| {
      buffer.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
      buffer.extend_from_slice(value);
    };

    field(b"filepack");
    field(b"");
    field(b"sha512");
    field(&Sha512::digest(&self.0));

    buffer
  }
}

impl AsRef<[u8]> for SerializedMessage {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}
