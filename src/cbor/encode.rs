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

impl<K, V> Encode for BTreeMap<K, V>
where
  K: Encode + PartialOrd,
  V: Encode,
{
  fn encode(&self, encoder: &mut Encoder) {
    let mut map = encoder.map::<&K>(self.len());
    for (key, value) in self {
      map.item(key, value);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bytes_encoding() {
    #[track_caller]
    fn case(value: &[u8], expected: &[u8]) {
      assert_eq!(value.encode_to_vec(), expected);
    }

    case(b"", &[0x40]);
    case(b"bar", &[0x43, 0x62, 0x61, 0x72]);
  }

  #[test]
  fn map_encoding() {
    let map = BTreeMap::from([("bar", 1u64), ("foo", 2u64)]);
    assert_eq!(
      map.encode_to_vec(),
      [
        0xA2, 0x63, 0x62, 0x61, 0x72, 0x01, 0x63, 0x66, 0x6F, 0x6F, 0x02,
      ],
    );
  }

  #[test]
  fn str_encoding() {
    #[track_caller]
    fn case(value: &str, expected: &[u8]) {
      assert_eq!(value.encode_to_vec(), expected);
    }

    case("", &[0x60]);
    case("foo", &[0x63, 0x66, 0x6F, 0x6F]);
  }

  #[test]
  fn u64_encoding() {
    #[track_caller]
    fn case(value: u64, expected: &[u8]) {
      assert_eq!(value.encode_to_vec(), expected);
    }

    case(0, &[0x00]);
    case(24, &[0x18, 0x18]);
    case(256, &[0x19, 0x01, 0x00]);
  }

  #[test]
  fn u8_encoding() {
    assert_eq!(100u8.encode_to_vec(), 100u64.encode_to_vec());
  }
}
