use super::*;

#[derive(Default)]
pub struct Encoder {
  buffer: VecDeque<u8>,
}

impl Encoder {
  pub fn array(&mut self) -> ArrayEncoder<'_> {
    ArrayEncoder::new(self)
  }

  pub fn boolean(&mut self, boolean: bool) {
    self.integer(u64::from(boolean));
  }

  pub fn bytes(&mut self, bytes: &[u8]) {
    let end = self.buffer.len();
    for &byte in bytes.iter().rev() {
      self.buffer.push_front(byte);
    }
    self.head(end);
  }

  pub fn finish(self) -> Vec<u8> {
    Vec::from(self.buffer)
  }

  pub(crate) fn head(&mut self, end: usize) {
    let len = self.buffer.len() - end;
    let head = Head::new(len, self.buffer.front().copied());
    match head {
      Head::Small => {}
      Head::Medium(len) => self.buffer.push_front((0x80 + len).try_into().unwrap()),
      Head::Large(count) => {
        for &byte in len.to_le_bytes()[..count].iter().rev() {
          self.buffer.push_front(byte);
        }
        self.buffer.push_front((0xEF + count).try_into().unwrap());
      }
      Head::Reserved(value) => self.buffer.push_front(value),
    }
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

  pub(crate) fn len(&self) -> usize {
    self.buffer.len()
  }

  pub fn map<K: Encode + PartialOrd>(&mut self) -> MapEncoder<'_, K> {
    MapEncoder::new(self)
  }

  pub fn new() -> Self {
    Self::default()
  }

  pub fn signed_integer(&mut self, integer: i64) {
    self.integer(((integer << 1) ^ (integer >> 63)).cast_unsigned());
  }

  pub fn text(&mut self, text: &str) {
    self.bytes(text.as_bytes());
  }
}
