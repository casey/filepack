use super::*;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Decoded<T> {
  Known(T),
  Unknown(Vec<u8>),
}

impl<T> Decoded<T> {
  pub fn known(&self) -> Option<&T> {
    match self {
      Self::Known(value) => Some(value),
      Self::Unknown(_) => None,
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
  fn encode(&self, encoder: &mut Encoder) {
    match self {
      Self::Known(value) => value.encode(encoder),
      Self::Unknown(bytes) => encoder.bytes(bytes),
    }
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

  #[derive(Debug, Decode, Encode, Eq, PartialEq, Serialize)]
  #[deco(strict)]
  struct Foo {
    #[n(1)]
    foo: u64,
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
