use super::*;

pub struct MapEncoder<'a, K> {
  encoder: &'a mut Encoder,
  end: usize,
  last: Option<K>,
}

impl<'a, K: Encode + PartialOrd> MapEncoder<'a, K> {
  pub fn finish(self) {
    self.encoder.head(self.end);
  }

  pub fn item(&mut self, key: K, value: impl Encode) {
    self.item_with(key, &value, Encode::encode);
  }

  pub fn item_with<V>(&mut self, key: K, value: &V, encode: impl FnOnce(&V, &mut Encoder)) {
    if let Some(last) = &self.last {
      assert!(key < *last, "out of order key");
    }

    encode(value, self.encoder);
    key.encode(self.encoder);

    self.last = Some(key);
  }

  pub(crate) fn new(encoder: &'a mut Encoder) -> Self {
    let end = encoder.len();
    Self {
      encoder,
      end,
      last: None,
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

#[cfg(test)]
mod tests {
  use {
    super::*,
    std::panic::{UnwindSafe, catch_unwind},
  };

  struct Foreign(u64);

  fn case(f: impl Fn() + UnwindSafe, expected: &str) {
    assert_eq!(
      *catch_unwind(f).unwrap_err().downcast::<&str>().unwrap(),
      expected
    );
  }

  fn encode_foreign(value: &Foreign, encoder: &mut Encoder) {
    (value.0 + 1).encode(encoder);
  }

  #[test]
  fn item_with() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>();
    map.item_with(0, &Foreign(42), encode_foreign);
    map.finish();
    assert_eq!(encoder.finish(), vec![0x82, 0x00, 0x2b]);
  }

  #[test]
  fn items() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>();
    map.item(1, 2u64);
    map.item(0, 1u64);
    map.finish();
    assert_eq!(encoder.finish(), vec![0x84, 0x00, 0x01, 0x01, 0x02]);
  }

  #[test]
  fn optional_item_none() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>();
    map.optional_item(0, None::<u64>);
    map.finish();
    assert_eq!(encoder.finish(), vec![0x80]);
  }

  #[test]
  fn optional_item_some() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>();
    map.optional_item(0, Some(42u64));
    map.finish();
    assert_eq!(encoder.finish(), vec![0x82, 0x00, 0x2a]);
  }

  #[test]
  fn optional_item_with_none() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>();
    map.optional_item_with(0, None::<&Foreign>, encode_foreign);
    map.finish();
    assert_eq!(encoder.finish(), vec![0x80]);
  }

  #[test]
  fn optional_item_with_some() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>();
    map.optional_item_with(0, Some(&Foreign(42)), encode_foreign);
    map.finish();
    assert_eq!(encoder.finish(), vec![0x82, 0x00, 0x2b]);
  }

  #[test]
  fn out_of_order() {
    case(
      || {
        let mut encoder = Encoder::new();
        let mut map = encoder.map::<u64>();
        map.item(1, 2u64);
        map.item(2, 1u64);
      },
      "out of order key",
    );
  }
}
