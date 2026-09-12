use super::*;

pub trait Encode {
  fn encode(&self, encoder: &mut Encoder);

  fn encode_to_vec(&self) -> Vec<u8> {
    let mut encoder = Encoder::new();
    self.encode(&mut encoder);
    encoder.finish()
  }
}

impl<T: ?Sized> Encode for &T
where
  T: Encode,
{
  fn encode(&self, encoder: &mut Encoder) {
    T::encode(self, encoder);
  }
}

impl<K, V> Encode for BTreeMap<K, V>
where
  K: Encode + PartialOrd,
  V: Encode,
{
  fn encode(&self, encoder: &mut Encoder) {
    let mut map = encoder.map::<&K>();
    for (key, value) in self.iter().rev() {
      map.item(key, value);
    }
    map.finish();
  }
}

impl Encode for bool {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.boolean(*self);
  }
}

impl Encode for String {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_str().encode(encoder);
  }
}

impl Encode for Vec<u8> {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_slice().encode(encoder);
  }
}

impl<T: Encode> Encode for Vec<T> {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_slice().encode(encoder);
  }
}

impl Encode for i32 {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.signed_integer((*self).into());
  }
}

impl Encode for i64 {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.signed_integer(*self);
  }
}

impl Encode for str {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.text(self);
  }
}

impl Encode for u64 {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.integer(*self);
  }
}

impl Encode for usize {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.integer(self.into_u64());
  }
}

impl Encode for [u8] {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.bytes(self);
  }
}

impl<T: Encode> Encode for [T] {
  fn encode(&self, encoder: &mut Encoder) {
    let mut array = encoder.array();
    for element in self.iter().rev() {
      array.element(element);
    }
    array.finish();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn array() {
    assert_deco(Vec::<u64>::new(), "80");
    assert_deco(vec![1u64, 2u64], "820102");
  }

  #[test]
  fn boolean() {
    assert_deco(false, "00");
    assert_deco(true, "01");
  }

  #[test]
  fn bytes() {
    assert_deco(Vec::<u8>::new(), "80");
    assert_deco(b"bar".to_vec(), "83626172");
  }

  #[test]
  fn i32() {
    assert_deco(0i32, "00");
    assert_deco(-1i32, "01");
    assert_deco(i32::MAX, "84feffffff");
    assert_deco(i32::MIN, "84ffffffff");
  }

  #[test]
  fn i64() {
    assert_deco(0i64, "00");
    assert_deco(-1i64, "01");
    assert_deco(1i64, "02");
    assert_deco(63i64, "7e");
    assert_deco(-64i64, "7f");
    assert_deco(64i64, "8180");
    assert_deco(-65i64, "8181");
    assert_deco(127i64, "81fe");
    assert_deco(-128i64, "81ff");
    assert_deco(128i64, "820001");
    assert_deco(-129i64, "820101");
    assert_deco(i64::MAX, "88feffffffffffffff");
    assert_deco(i64::MIN, "88ffffffffffffffff");
  }

  #[test]
  fn map() {
    assert_deco(
      BTreeMap::from([("bar".to_string(), 1u64), ("foo".to_string(), 2u64)]),
      "8a836261720183666f6f02",
    );
  }

  #[test]
  fn nested_collections() {
    assert_deco(
      vec![Vec::<String>::new(), vec![String::from("foo")]],
      "86808483666f6f",
    );
    assert_deco(
      BTreeMap::from([(String::from("foo"), vec![0u64, 128])]),
      "8883666f6f83008180",
    );
  }

  #[test]
  fn string() {
    assert_deco(String::new(), "80");
    assert_deco(String::from("foo"), "83666f6f");
  }

  #[test]
  fn u64() {
    assert_deco(0u64, "00");
    assert_deco(24u64, "18");
    assert_deco(127u64, "7f");
    assert_deco(128u64, "8180");
    assert_deco(255u64, "81ff");
    assert_deco(u64::MAX, "88ffffffffffffffff");
    assert_deco(256u64, "820001");
  }

  #[test]
  fn usize() {
    assert_deco(42usize, "2a");
  }
}
