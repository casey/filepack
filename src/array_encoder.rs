use super::*;

pub struct ArrayEncoder<'a> {
  encoder: &'a mut Encoder,
  end: usize,
}

impl<'a> ArrayEncoder<'a> {
  pub fn element(&mut self, value: impl Encode) {
    value.encode(self.encoder);
  }

  pub fn encoder(&mut self) -> &mut Encoder {
    self.encoder
  }

  pub fn finish(self) {
    self.encoder.head(self.end);
  }

  pub(crate) fn new(encoder: &'a mut Encoder) -> Self {
    let end = encoder.len();
    Self { encoder, end }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn finish() {
    let mut encoder = Encoder::new();
    encoder.integer(128);
    let mut array = encoder.array();
    array.element(0u64);
    array.finish();
    encoder.array().finish();
    assert_eq!(encoder.finish(), [0x80, 0x00, 0x81, 0x80]);
  }
}
