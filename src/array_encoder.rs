use super::*;

pub struct ArrayEncoder<'a> {
  encoder: &'a mut Encoder,
  remaining: u64,
}

impl<'a> ArrayEncoder<'a> {
  pub fn item(&mut self, value: impl Encode) {
    self.item_with(|encoder| value.encode(encoder));
  }

  pub fn item_with(&mut self, encode: impl FnOnce(&mut Encoder)) {
    assert!(self.remaining > 0, "too many items");
    encode(self.encoder);
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

#[cfg(test)]
mod tests {
  use {
    super::*,
    std::panic::{UnwindSafe, catch_unwind},
  };

  fn case(f: impl Fn() + UnwindSafe, expected: &str) {
    assert_eq!(
      *catch_unwind(f).unwrap_err().downcast::<&str>().unwrap(),
      expected
    );
  }

  #[test]
  fn item() {
    let mut encoder = Encoder::new();
    let mut array = encoder.array(2);
    array.item(0u64);
    array.item("foo");
    drop(array);
    assert_eq!(encoder.finish(), vec![0x82, 0x00, 0x63, 0x66, 0x6f, 0x6f]);
  }

  #[test]
  fn item_with() {
    let mut encoder = Encoder::new();
    let mut array = encoder.array(1);
    array.item_with(|encoder| 42u64.encode(encoder));
    drop(array);
    assert_eq!(encoder.finish(), vec![0x81, 0x18, 0x2a]);
  }

  #[test]
  fn too_few_items() {
    case(
      || {
        let mut encoder = Encoder::new();
        let mut array = ArrayEncoder::new(&mut encoder, 2);
        array.item(0u64);
      },
      "too few items",
    );
  }

  #[test]
  fn too_many_items() {
    case(
      || {
        let mut encoder = Encoder::new();
        let mut array = encoder.array(1);
        array.item(0u64);
        array.item(1u64);
      },
      "too many items",
    );
  }
}
