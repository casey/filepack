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
    let mut map = encoder.map::<&K>(self.len().into_u64());
    for (key, value) in self {
      map.item(key, value);
    }
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
    let mut array = encoder.array(self.len().into_u64());
    for item in self {
      array.item(item);
    }
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
  fn bytes() {
    assert_cbor(Vec::<u8>::new(), "40");
    assert_cbor(b"bar".to_vec(), "43626172");
  }

  #[test]
  fn i32() {
    assert_cbor(0i32, "00");
    assert_cbor(-1i32, "20");
    assert_cbor(i32::MAX, "1a7fffffff");
    assert_cbor(i32::MIN, "3a7fffffff");
  }

  #[test]
  fn i64() {
    assert_cbor(0i64, "00");
    assert_cbor(-1i64, "20");
    assert_cbor(i64::MAX, "1b7fffffffffffffff");
    assert_cbor(i64::MIN, "3b7fffffffffffffff");
  }

  #[test]
  fn map() {
    assert_cbor(
      BTreeMap::from([("bar".to_string(), 1u64), ("foo".to_string(), 2u64)]),
      "a2636261720163666f6f02",
    );
  }

  #[test]
  fn string() {
    assert_cbor(String::new(), "60");
    assert_cbor(String::from("foo"), "63666f6f");
  }

  #[test]
  fn u64() {
    assert_cbor(0u64, "00");
    assert_cbor(24u64, "1818");
    assert_cbor(256u64, "190100");
  }

  #[test]
  fn usize() {
    assert_cbor(42usize, "182a");
  }
}
