pub struct SerializedMessage(pub(crate) Vec<u8>);

impl SerializedMessage {
  pub(crate) fn ssh_signed_data(&self) -> Vec<u8> {
    use sha2::{Digest, Sha512};

    let hash = Sha512::digest(&self.0);

    let mut buf = b"SSHSIG".to_vec();

    let mut field = |bytes: &[u8]| {
      buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
      buf.extend_from_slice(bytes);
    };

    field(b"filepack");
    field(b"");
    field(b"sha512");
    field(&hash);

    buf
  }
}

impl AsRef<[u8]> for SerializedMessage {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}
