use super::*;

pub(crate) struct Encoder {
  buffer: Vec<u8>,
}

impl Encoder {
  pub(crate) fn new() -> Self {
    Self { buffer: Vec::new() }
  }

  pub(crate) fn map<K: Encode + PartialOrd>(&mut self, length: usize) -> MapEncoder<K> {
    MapEncoder::new(self, length.into_u64())
  }

  pub(crate) fn head(&mut self, head: Head) {
    let i = self.buffer.len();

    if head.value < 24 {
      self.buffer.push(head.value.try_into().unwrap());
    } else if let Ok(n) = u8::try_from(head.value) {
      self.buffer.push(24);
      self.buffer.push(n);
    } else if let Ok(n) = u16::try_from(head.value) {
      self.buffer.push(25);
      self.buffer.extend(n.to_be_bytes());
    } else if let Ok(n) = u32::try_from(head.value) {
      self.buffer.push(26);
      self.buffer.extend(n.to_be_bytes());
    } else {
      self.buffer.push(27);
      self.buffer.extend(head.value.to_be_bytes());
    }

    self.buffer[i] |= head.major_type.value() << 5;
  }

  pub(crate) fn text(&mut self, text: &str) {
    self.head(MajorType::Text.head(text.len().into_u64()));
    self.buffer.extend(text.as_bytes());
  }

  pub(crate) fn integer(&mut self, integer: u64) {
    self.head(MajorType::Integer.head(integer));
  }

  pub(crate) fn bytes(&mut self, bytes: &[u8]) {
    self.head(MajorType::Bytes.head(bytes.len().into_u64()));
    self.buffer.extend(bytes);
  }

  pub(crate) fn finish(self) -> Vec<u8> {
    self.buffer
  }
}
