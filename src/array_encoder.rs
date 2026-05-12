use super::*;

pub struct ArrayEncoder<'a> {
  encoder: &'a mut Encoder,
  remaining: u64,
}

impl<'a> ArrayEncoder<'a> {
  pub(crate) fn element(&mut self, value: impl Encode) {
    assert!(self.remaining > 0, "too many items");
    value.encode(self.encoder);
    self.remaining -= 1;
  }

  pub(crate) fn new(encoder: &'a mut Encoder, length: u64) -> Self {
    encoder.head(MajorType::Array.head(length));
    Self {
      encoder,
      remaining: length,
    }
  }
}

impl Drop for ArrayEncoder<'_> {
  fn drop(&mut self) {
    if !std::thread::panicking() {
      assert!(self.remaining == 0, "too few items");
    }
  }
}
