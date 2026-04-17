use super::*;

pub(crate) struct MapEncoder<'a, K> {
  encoder: &'a mut Encoder,
  last: Option<K>,
  remaining: u64,
}

impl<'a, K: Encode + PartialOrd> MapEncoder<'a, K> {
  pub(crate) fn item(&mut self, key: K, value: impl Encode) {
    assert!(self.remaining > 0, "too many items");

    if let Some(last) = &self.last {
      assert!(key > *last, "out of order key");
    }

    key.encode(self.encoder);
    value.encode(self.encoder);

    self.last = Some(key);
    self.remaining -= 1;
  }

  pub(crate) fn new(encoder: &'a mut Encoder, length: u64) -> Self {
    encoder.head(MajorType::Map.head(length));
    Self {
      encoder,
      last: None,
      remaining: length,
    }
  }
}

impl<K> Drop for MapEncoder<'_, K> {
  fn drop(&mut self) {
    if !std::thread::panicking() && self.remaining != 0 {
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
  fn out_of_order() {
    case(
      || {
        let mut encoder = Encoder::new();
        let mut map = encoder.map::<u8>(2);
        map.item(2, 1u8);
        map.item(1, 2u8);
      },
      "out of order key",
    );
  }

  #[test]
  fn too_few_items() {
    case(
      || {
        let mut encoder = Encoder::new();
        let mut map = MapEncoder::<u8>::new(&mut encoder, 2);
        map.item(0, 0u8);
      },
      "too few items",
    );
  }

  #[test]
  fn too_many_items() {
    case(
      || {
        let mut encoder = Encoder::new();
        let mut map = encoder.map::<u8>(1);
        map.item(0, 0u8);
        map.item(1, 1u8);
      },
      "too many items",
    );
  }
}
