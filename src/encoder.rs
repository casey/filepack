use super::*;

#[derive(Default)]
pub struct Encoder {
  buffer: VecDeque<u8>,
}

impl Encoder {
  pub(crate) fn array(&mut self) -> ArrayEncoder<'_> {
    ArrayEncoder::new(self)
  }

  pub(crate) fn boolean(&mut self, boolean: bool) {
    self.integer(u64::from(boolean));
  }

  pub fn bytes(&mut self, bytes: &[u8]) {
    for &byte in bytes.iter().rev() {
      self.buffer.push_front(byte);
    }
    self.head(bytes.len());
  }

  pub fn finish(self) -> Vec<u8> {
    Vec::from(self.buffer)
  }

  pub(crate) fn frame(buffer: Vec<u8>) -> Vec<u8> {
    let mut encoder = Self {
      buffer: buffer.into(),
    };
    encoder.head(encoder.buffer.len());
    encoder.finish()
  }

  pub(crate) fn head(&mut self, len: usize) {
    let head = Head::new(len, self.buffer.front().copied());
    match head {
      Head::Small => {}
      Head::Medium(len) => self.buffer.push_front((0x80 + len).try_into().unwrap()),
      Head::Large(count) => {
        for &byte in len.to_le_bytes()[..count].iter().rev() {
          self.buffer.push_front(byte);
        }
        self.buffer.push_front((0xef + count).try_into().unwrap());
      }
      Head::Reserved(_) => unreachable!(),
    }
  }

  pub(crate) fn integer(&mut self, integer: u64) {
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

  pub(crate) fn signed_integer(&mut self, integer: i64) {
    self.integer(((integer << 1) ^ (integer >> 63)).cast_unsigned());
  }

  pub fn text(&mut self, text: &str) {
    self.bytes(text.as_bytes());
  }
}
