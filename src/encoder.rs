use super::*;

#[derive(Default)]
pub struct Encoder {
  pub(crate) buffer: Vec<u8>,
}

impl Encoder {
  pub fn boolean(&mut self, boolean: bool) {
    self.integer(u64::from(boolean));
  }

  pub fn bytes(&mut self, bytes: &[u8]) {
    Head::from(bytes).encode(bytes, self);
    self.buffer.extend_from_slice(bytes);
  }

  pub fn finish(self) -> Vec<u8> {
    self.buffer
  }

  pub fn integer(&mut self, integer: u64) {
    let bytes = integer.to_le_bytes();
    let len = bytes
      .iter()
      .rposition(|&byte| byte != 0)
      .unwrap_or_default()
      + 1;
    self.bytes(&bytes[..len]);
  }

  pub fn new() -> Self {
    Self::default()
  }

  pub fn signed_integer(&mut self, integer: i64) {
    let bytes = integer.to_le_bytes();
    let mut len = bytes.len();
    while len > 1
      && ((bytes[len - 1] == 0 && bytes[len - 2] < 0x80)
        || (bytes[len - 1] == 0xFF && bytes[len - 2] >= 0x80))
    {
      len -= 1;
    }
    self.bytes(&bytes[..len]);
  }

  pub fn text(&mut self, text: &str) {
    self.bytes(text.as_bytes());
  }
}
