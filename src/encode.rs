use super::*;

pub trait Encode {
  fn deco_schema(registry: &mut deco::Registry) -> String;

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
  fn deco_schema(registry: &mut deco::Registry) -> String {
    T::deco_schema(registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    T::encode(self, encoder);
  }
}

impl<const N: usize> Encode for [u8; N] {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    deco::Schema::Bytes {
      length: Some(N.into_u64()),
    }
    .register(&format!("Bytes<{N}>"), registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    encoder.bytes(self);
  }
}

impl<const N: usize, const M: usize> Encode for [[u8; N]; M] {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    deco::Schema::Bytes {
      length: Some((N * M).into_u64()),
    }
    .register(&format!("Bytes<{}>", N * M), registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    encoder.bytes(self.as_flattened());
  }
}

impl<K, V> Encode for BTreeMap<K, V>
where
  K: Encode + PartialOrd,
  V: Encode,
{
  fn deco_schema(registry: &mut deco::Registry) -> String {
    let key = K::deco_schema(registry);
    let value = V::deco_schema(registry);
    let name = format!("Map<{key}, {value}>");
    deco::Schema::Map { key, value }.register(&name, registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    let mut map = encoder.map::<&K>();
    for (key, value) in self.iter().rev() {
      map.item(key, value);
    }
    map.finish();
  }
}

impl Encode for bool {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    deco::Schema::Boolean.register("Boolean", registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    encoder.boolean(*self);
  }
}

impl Encode for String {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    str::deco_schema(registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    self.as_str().encode(encoder);
  }
}

impl Encode for Vec<u8> {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    <[u8]>::deco_schema(registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    self.as_slice().encode(encoder);
  }
}

impl<T: Encode> Encode for Vec<T> {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    <[T]>::deco_schema(registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    self.as_slice().encode(encoder);
  }
}

impl Encode for i64 {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    deco::Schema::SignedInteger.register("SignedInteger", registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    encoder.signed_integer(*self);
  }
}

impl Encode for str {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    deco::Schema::String.register("String", registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    encoder.text(self);
  }
}

impl Encode for u64 {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    deco::Schema::UnsignedInteger.register("UnsignedInteger", registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    encoder.integer(*self);
  }
}

impl Encode for [u8] {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    deco::Schema::Bytes { length: None }.register("Bytes", registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    encoder.bytes(self);
  }
}

impl<T: Encode> Encode for [T] {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    let element = T::deco_schema(registry);
    let name = format!("Array<{element}>");
    deco::Schema::Array { element }.register(&name, registry)
  }

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
  fn array_schema() {
    let mut schemas = BTreeMap::new();

    assert_eq!(
      Vec::<u64>::deco_schema(&mut schemas),
      "Array<UnsignedInteger>"
    );

    assert_eq!(
      schemas,
      BTreeMap::from([
        (
          "Array<UnsignedInteger>".into(),
          deco::Schema::Array {
            element: "UnsignedInteger".into(),
          },
        ),
        ("UnsignedInteger".into(), deco::Schema::UnsignedInteger),
      ]),
    );
  }

  #[test]
  fn boolean() {
    assert_deco(false, "00");
    assert_deco(true, "01");
  }

  #[test]
  fn byte_array() {
    assert_deco([7u8], "07");
    assert_deco([0x80u8], "8180");
    assert_deco([1u8, 2], "820102");
  }

  #[test]
  fn byte_array_schema() {
    let mut schemas = BTreeMap::new();

    assert_eq!(<[u8; 32]>::deco_schema(&mut schemas), "Bytes<32>");
    assert_eq!(<[[u8; 32]; 2]>::deco_schema(&mut schemas), "Bytes<64>");

    assert_eq!(
      schemas,
      BTreeMap::from([
        ("Bytes<32>".into(), deco::Schema::Bytes { length: Some(32) }),
        ("Bytes<64>".into(), deco::Schema::Bytes { length: Some(64) }),
      ]),
    );
  }

  #[test]
  fn bytes() {
    assert_deco(Vec::<u8>::new(), "80");
    assert_deco(b"bar".to_vec(), "83626172");
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
  fn map_schema() {
    let mut schemas = BTreeMap::new();

    assert_eq!(
      BTreeMap::<String, u64>::deco_schema(&mut schemas),
      "Map<String, UnsignedInteger>"
    );

    assert_eq!(
      schemas,
      BTreeMap::from([
        (
          "Map<String, UnsignedInteger>".into(),
          deco::Schema::Map {
            key: "String".into(),
            value: "UnsignedInteger".into(),
          },
        ),
        ("String".into(), deco::Schema::String),
        ("UnsignedInteger".into(), deco::Schema::UnsignedInteger),
      ]),
    );
  }

  #[test]
  fn nested_byte_array() {
    assert_deco([[1u8], [2]], "820102");
    assert_deco([[1u8, 2], [3, 4]], "8401020304");
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
}
