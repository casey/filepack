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
    let mut map = MapEncoder::<&K>::new();
    for (key, value) in self {
      map.item(key, value);
    }
    encoder.bytes(&map.finish());
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
    let mut array = Encoder::new();
    for item in self {
      item.encode(&mut array);
    }
    encoder.bytes(&array.finish());
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn array() {
    assert_cbor(Vec::<u64>::new(), "80");
    assert_cbor(vec![1u64, 2u64], "820102");
  }

  #[test]
  fn boolean() {
    assert_cbor(false, "00");
    assert_cbor(true, "01");
  }

  #[test]
  fn bytes() {
    assert_cbor(Vec::<u8>::new(), "80");
    assert_cbor(b"bar".to_vec(), "83626172");
  }

  #[test]
  fn i32() {
    assert_cbor(0i32, "00");
    assert_cbor(-1i32, "81ff");
    assert_cbor(i32::MAX, "84ffffff7f");
    assert_cbor(i32::MIN, "8400000080");
  }

  #[test]
  fn i64() {
    assert_cbor(0i64, "00");
    assert_cbor(-1i64, "81ff");
    assert_cbor(127i64, "7f");
    assert_cbor(128i64, "828000");
    assert_cbor(255i64, "82ff00");
    assert_cbor(-128i64, "8180");
    assert_cbor(-129i64, "827fff");
    assert_cbor(i64::MAX, "88ffffffffffffff7f");
    assert_cbor(i64::MIN, "880000000000000080");
  }

  #[test]
  fn map() {
    assert_cbor(
      BTreeMap::from([("bar".to_string(), 1u64), ("foo".to_string(), 2u64)]),
      "8a836261720183666f6f02",
    );
  }

  #[test]
  fn nested_collections() {
    assert_cbor(
      vec![Vec::<String>::new(), vec![String::from("foo")]],
      "86808483666f6f",
    );
    assert_cbor(
      BTreeMap::from([(String::from("foo"), vec![0u64, 128])]),
      "8883666f6f83008180",
    );
  }

  #[test]
  fn string() {
    assert_cbor(String::new(), "80");
    assert_cbor(String::from("foo"), "83666f6f");
  }

  #[test]
  fn u64() {
    assert_cbor(0u64, "00");
    assert_cbor(24u64, "18");
    assert_cbor(127u64, "7f");
    assert_cbor(128u64, "8180");
    assert_cbor(255u64, "81ff");
    assert_cbor(u64::MAX, "88ffffffffffffffff");
    assert_cbor(256u64, "820001");
  }

  #[test]
  fn usize() {
    assert_cbor(42usize, "2a");
  }
}
