use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Decoded<T> {
  Known(T),
  Unknown(Vec<u8>),
}

impl<T> Decoded<T> {
  pub fn is_unknown(&self) -> bool {
    self.known().is_none()
  }

  pub fn known(&self) -> Option<&T> {
    match self {
      Self::Known(value) => Some(value),
      Self::Unknown(_) => None,
    }
  }
}

impl<T: Display> Display for Decoded<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Known(value) => write!(f, "{value}"),
      Self::Unknown(_) => write!(f, "unknown"),
    }
  }
}

impl<'a, T: Decode<'a>> Decode<'a> for Decoded<T> {
  fn decode(decoder: &mut Decoder<'a>) -> DecodeResult<Self> {
    let mut attempt = decoder.clone();

    match T::decode(&mut attempt) {
      Ok(value) => {
        *decoder = attempt;
        Ok(Self::Known(value))
      }
      Err(DecodeError::Unknown { .. }) => Ok(Self::Unknown(decoder.bytes()?.to_vec())),
      Err(error) => Err(error),
    }
  }
}

impl<T: Encode> Encode for Decoded<T> {
  fn deco_schema(registry: &mut deco::Registry) -> String {
    T::deco_schema(registry)
  }

  fn encode(&self, encoder: &mut Encoder) {
    match self {
      Self::Known(value) => value.encode(encoder),
      Self::Unknown(bytes) => encoder.bytes(bytes),
    }
  }
}

impl<T: json::Json> json::Json for Decoded<T> {
  fn json_schema(registry: &mut json::Registry) -> String {
    T::json_schema(registry)
  }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Decoded<T> {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    T::deserialize(deserializer).map(Self::Known)
  }
}

impl<T: Serialize> Serialize for Decoded<T> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Self::Known(value) => value.serialize(serializer),
      Self::Unknown(_) => Err(serde::ser::Error::custom(
        "unknown value cannot be serialized",
      )),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Foo.
  #[derive(Debug, Decode, Encode, PartialEq, Serialize)]
  #[deco(strict)]
  struct Foo {
    /// Foo.
    #[n(1)]
    foo: u64,
  }

  #[test]
  fn display() {
    assert_eq!(Decoded::Known(1).to_string(), "1");
    assert_eq!(Decoded::<u64>::Unknown(vec![99]).to_string(), "unknown");
  }

  #[test]
  fn known() {
    let foo = Foo { foo: 1 };
    let bytes = foo.encode_to_vec();
    let decoded = Decoded::<Foo>::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded, Decoded::Known(Foo { foo: 1 }));
    assert_eq!(decoded.encode_to_vec(), bytes);
    assert_eq!(serde_json::to_string(&decoded).unwrap(), r#"{"foo":1}"#);
  }

  #[test]
  fn schema() {
    let mut schemas = BTreeMap::new();

    assert_eq!(Decoded::<u64>::deco_schema(&mut schemas), "UnsignedInteger");

    assert_eq!(
      schemas,
      BTreeMap::from([("UnsignedInteger".into(), deco::Schema::of::<u64>())]),
    );
  }

  #[test]
  fn unknown() {
    let bytes = with_unknown_field(Foo { foo: 1 });
    let decoded = Decoded::<Foo>::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded, Decoded::Unknown(bytes[1..].to_vec()));
    assert_eq!(decoded.encode_to_vec(), bytes);
    assert_eq!(
      serde_json::to_string(&decoded).unwrap_err().to_string(),
      "unknown value cannot be serialized",
    );
  }
}
