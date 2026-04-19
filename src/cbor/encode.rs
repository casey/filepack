use super::*;

pub(crate) trait Encode {
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

impl Encode for str {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.text(self);
  }
}

impl Encode for u8 {
  fn encode(&self, encoder: &mut Encoder) {
    encoder.integer((*self).into());
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bytes() {
    assert_encoding(Vec::<u8>::new(), &[0x40]);
    assert_encoding(b"bar".to_vec(), &[0x43, 0x62, 0x61, 0x72]);
  }

  #[test]
  fn map() {
    assert_encoding(
      BTreeMap::from([("bar".to_string(), 1u64), ("foo".to_string(), 2u64)]),
      &[
        0xA2, 0x63, 0x62, 0x61, 0x72, 0x01, 0x63, 0x66, 0x6F, 0x6F, 0x02,
      ],
    );
  }

  #[test]
  fn string() {
    assert_encoding(String::new(), &[0x60]);
    assert_encoding(String::from("foo"), &[0x63, 0x66, 0x6F, 0x6F]);
  }

  #[test]
  fn u64() {
    assert_encoding(0u64, &[0x00]);
    assert_encoding(24u64, &[0x18, 0x18]);
    assert_encoding(256u64, &[0x19, 0x01, 0x00]);
  }

  #[test]
  fn u8() {
    assert_encoding(100u8, &[0x18, 0x64]);
  }

  #[test]
  fn usize() {
    assert_encoding(42usize, &[0x18, 0x2A]);
  }
}
