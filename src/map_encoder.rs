use super::*;

pub struct MapEncoder<'a, K> {
  encoder: &'a mut Encoder,
  last: Option<K>,
  remaining: u64,
}

impl<'a, K: Encode + PartialOrd> MapEncoder<'a, K> {
  pub fn item(&mut self, key: K, value: impl Encode) {
    self.item_with(key, &value, Encode::encode);
  }

  pub fn item_with<V>(&mut self, key: K, value: &V, encode: impl FnOnce(&V, &mut Encoder)) {
    assert!(self.remaining > 0, "too many items");

    if let Some(last) = &self.last {
      assert!(key > *last, "out of order key");
    }

    key.encode(self.encoder);
    encode(value, self.encoder);

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

  pub fn optional_item(&mut self, key: K, value: Option<impl Encode>) {
    if let Some(value) = value {
      self.item(key, value);
    }
  }

  pub fn optional_item_with<V>(
    &mut self,
    key: K,
    value: Option<&V>,
    encode: impl FnOnce(&V, &mut Encoder),
  ) {
    if let Some(value) = value {
      self.item_with(key, value, encode);
    }
  }
}

impl<K> Drop for MapEncoder<'_, K> {
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

  struct Foreign(u64);

  fn encode_foreign(value: &Foreign, encoder: &mut Encoder) {
    (value.0 + 1).encode(encoder);
  }

  fn case(f: impl Fn() + UnwindSafe, expected: &str) {
    assert_eq!(
      *catch_unwind(f).unwrap_err().downcast::<&str>().unwrap(),
      expected
    );
  }

  #[test]
  fn item_with() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u8>(1);
    map.item_with(0, &Foreign(42), encode_foreign);
    drop(map);
    assert_eq!(encoder.finish(), vec![0xa1, 0x00, 0x18, 0x2b]);
  }

  #[test]
  fn optional_item_none() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u8>(0);
    map.optional_item(0, None::<u8>);
    drop(map);
    assert_eq!(encoder.finish(), vec![0xa0]);
  }

  #[test]
  fn optional_item_some() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u8>(1);
    map.optional_item(0, Some(42u8));
    drop(map);
    assert_eq!(encoder.finish(), vec![0xa1, 0x00, 0x18, 0x2a]);
  }

  #[test]
  fn optional_item_with_none() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u8>(0);
    map.optional_item_with(0, None::<&Foreign>, encode_foreign);
    drop(map);
    assert_eq!(encoder.finish(), vec![0xa0]);
  }

  #[test]
  fn optional_item_with_some() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u8>(1);
    map.optional_item_with(0, Some(&Foreign(42)), encode_foreign);
    drop(map);
    assert_eq!(encoder.finish(), vec![0xa1, 0x00, 0x18, 0x2b]);
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
