use super::*;

#[test]
fn all_optional_all_none() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: Option<u64>,
    #[n(1)]
    baz: Option<String>,
  }

  assert_cbor(
    Foo {
      bar: None,
      baz: None,
    },
    "a0",
  );
}

#[test]
fn all_optional_all_some() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: Option<u64>,
    #[n(1)]
    baz: Option<String>,
  }

  assert_cbor(
    Foo {
      bar: Some(1),
      baz: Some("foo".into()),
    },
    "a200010163666f6f",
  );
}

#[test]
fn all_optional_mixed() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: Option<u64>,
    #[n(1)]
    baz: Option<String>,
  }

  assert_cbor(
    Foo {
      bar: Some(1),
      baz: None,
    },
    "a10001",
  );

  assert_cbor(
    Foo {
      bar: None,
      baz: Some("foo".into()),
    },
    "a10163666f6f",
  );
}

#[test]
fn all_required() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: u64,
    #[n(1)]
    baz: String,
  }

  assert_cbor(
    Foo {
      bar: 42,
      baz: "foo".into(),
    },
    "a200182a0163666f6f",
  );
}

#[test]
fn decode_from_str() {
  #[derive(Debug, DecodeFromStr, PartialEq)]
  struct Foo(String);

  #[derive(Debug, Snafu)]
  #[snafu(display("bar error"))]
  struct FooError;

  impl FromStr for Foo {
    type Err = FooError;

    fn from_str(s: &str) -> Result<Self, FooError> {
      if s == "foo" {
        Ok(Foo(s.to_string()))
      } else {
        Err(FooError)
      }
    }
  }

  assert_eq!(
    Foo::decode_from_slice(&[0x63, 0x66, 0x6f, 0x6f]).unwrap(),
    Foo("foo".to_string()),
  );

  let err = Foo::decode_from_slice(&[0x63, 0x62, 0x61, 0x72]).unwrap_err();

  assert_matches!(
    err,
    DecodeError::FromStr {
      name: "Foo",
      ref source,
    } if source.to_string() == "bar error",
  );
}

#[test]
fn decode_with_optional() {
  fn decode_offset(decoder: &mut Decoder) -> Result<u64, DecodeError> {
    Ok(decoder.integer()? + 1)
  }

  #[derive(Debug, Decode, PartialEq)]
  struct Foo {
    #[cbor(decode_with = decode_offset)]
    #[n(0)]
    bar: Option<u64>,
  }

  assert_eq!(
    Foo::decode_from_slice(&[0xa1, 0x00, 0x18, 0x63]).unwrap(),
    Foo { bar: Some(100) },
  );

  assert_eq!(Foo::decode_from_slice(&[0xa0]).unwrap(), Foo { bar: None });
}

#[test]
fn decode_with_required() {
  fn decode_offset(decoder: &mut Decoder) -> Result<u64, DecodeError> {
    Ok(decoder.integer()? + 1)
  }

  #[derive(Debug, Decode, PartialEq)]
  struct Foo {
    #[cbor(decode_with = decode_offset)]
    #[n(0)]
    bar: u64,
  }

  assert_eq!(
    Foo::decode_from_slice(&[0xa1, 0x00, 0x18, 0x63]).unwrap(),
    Foo { bar: 100 },
  );
}

#[test]
fn encode_display() {
  #[derive(EncodeDisplay)]
  struct Foo;

  impl Display for Foo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      write!(f, "foo")
    }
  }

  assert_eq!(Foo.encode_to_vec(), [0x63, 0x66, 0x6f, 0x6f]);
}

#[test]
fn encode_with_optional() {
  struct Foreign(u64);

  fn encode_foreign(value: &Foreign, encoder: &mut Encoder) {
    (value.0 + 1).encode(encoder);
  }

  #[derive(Encode)]
  struct Foo {
    #[cbor(encode_with = encode_foreign)]
    #[n(0)]
    bar: Option<Foreign>,
  }

  assert_eq!(
    Foo {
      bar: Some(Foreign(99)),
    }
    .encode_to_vec(),
    [0xa1, 0x00, 0x18, 0x64],
  );

  assert_eq!(Foo { bar: None }.encode_to_vec(), [0xa0]);
}

#[test]
fn encode_with_required() {
  struct Foreign(u64);

  fn encode_foreign(value: &Foreign, encoder: &mut Encoder) {
    (value.0 + 1).encode(encoder);
  }

  #[derive(Encode)]
  struct Foo {
    #[cbor(encode_with = encode_foreign)]
    #[n(0)]
    bar: Foreign,
  }

  assert_eq!(
    Foo { bar: Foreign(99) }.encode_to_vec(),
    [0xa1, 0x00, 0x18, 0x64],
  );
}

#[test]
fn enum_array_invalid_discriminant() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
    #[n(1)]
    Baz {
      #[n(0)]
      baz: u64,
    },
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x82, 0x05, 0xa0]),
    Err(DecodeError::InvalidDiscriminant {
      discriminant: 5,
      name: "Foo",
    }),
  );
}

#[test]
fn enum_array_missing_element() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(0)]
      bar: u64,
    },
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x81, 0x00]),
    Err(DecodeError::MissingElement),
  );
}

#[test]
fn enum_array_unconsumed_elements() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(0)]
      bar: u64,
    },
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x83, 0x00, 0xa1, 0x00, 0x05, 0x00]),
    Err(DecodeError::UnconsumedElements),
  );
}

#[test]
fn enum_invalid_discriminant() {
  #[derive(Debug, Decode)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  #[track_caller]
  fn case(bytes: &[u8], expected: u64) {
    assert_matches!(
      Foo::decode_from_slice(bytes),
      Err(DecodeError::InvalidDiscriminant {
        discriminant,
        name: "Foo",
      }) if discriminant == expected,
    );
  }

  case(&[0x01], 1);
  case(&256u64.encode_to_vec(), 256);
}

#[test]
fn enum_mixed() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
    #[n(1)]
    Baz {
      #[n(0)]
      baz: u64,
    },
  }

  assert_cbor(Foo::Bar, "00");
  assert_cbor(Foo::Baz { baz: 99 }, "8201a1001863");
}

#[test]
fn enum_named_field() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(0)]
      bar: u64,
      #[n(1)]
      baz: String,
    },
  }

  assert_cbor(
    Foo::Bar {
      bar: 42,
      baz: "foo".into(),
    },
    "8200a200182a0163666f6f",
  );
}

#[test]
fn enum_named_field_optional() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(0)]
      bar: Option<u64>,
      #[n(1)]
      baz: u64,
    },
  }

  assert_cbor(
    Foo::Bar {
      bar: Some(1),
      baz: 2,
    },
    "8200a200010102",
  );

  assert_cbor(Foo::Bar { bar: None, baz: 2 }, "8200a10102");
}

#[test]
fn enum_round_trip() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
    #[n(1)]
    Baz,
  }

  assert_cbor(Foo::Bar, "00");
  assert_cbor(Foo::Baz, "01");
}

#[test]
fn enum_unexpected_type() {
  #[derive(Debug, Decode)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  assert_matches!(
    Foo::decode_from_slice(&"foo".encode_to_vec()),
    Err(DecodeError::UnexpectedVariantType {
      actual: MajorType::Text,
    }),
  );
}

#[test]
fn enum_variant_encode_with() {
  struct Foreign(u64);

  fn encode_foreign(value: &Foreign, encoder: &mut Encoder) {
    (value.0 + 1).encode(encoder);
  }

  #[derive(Encode)]
  enum Foo {
    #[n(0)]
    Bar {
      #[cbor(encode_with = encode_foreign)]
      #[n(0)]
      bar: Foreign,
    },
  }

  assert_eq!(
    Foo::Bar { bar: Foreign(99) }.encode_to_vec(),
    [0x82, 0x00, 0xa1, 0x00, 0x18, 0x64],
  );
}

#[test]
fn mixed_required_and_optional() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: Option<u64>,
    #[n(1)]
    baz: String,
  }

  assert_cbor(
    Foo {
      bar: Some(1),
      baz: "foo".into(),
    },
    "a200010163666f6f",
  );

  assert_cbor(
    Foo {
      bar: None,
      baz: "foo".into(),
    },
    "a10163666f6f",
  );
}

#[test]
fn single_field() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: u64,
  }

  assert_cbor(Foo { bar: 99 }, "a1001863");
}

#[test]
fn transparent_named() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[cbor(transparent)]
  struct Foo {
    bar: String,
  }

  assert_cbor(Foo { bar: "foo".into() }, "63666f6f");
  assert_cbor_eq(Foo { bar: "foo".into() }, "foo");
}

#[test]
fn transparent_newtype() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[cbor(transparent)]
  struct Foo(u64);

  assert_cbor(Foo(99), "1863");
  assert_cbor_eq(Foo(99), 99u64);
}

#[test]
fn validate() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[cbor(transparent, validate)]
  struct Foo(u64);

  impl Validate for Foo {
    fn validate(&self) -> Result<(), DecodeError> {
      ensure!(
        self.0 != 0,
        decode_error::UnexpectedValue {
          actual: self.0.to_string(),
          expected: "nonzero integer",
        }
      );
      Ok(())
    }
  }

  assert_cbor(Foo(99), "1863");

  assert_matches!(
    Foo::decode_from_slice(&[0x00]),
    Err(DecodeError::UnexpectedValue {
      ref actual,
      expected: "nonzero integer",
    }) if actual == "0",
  );
}
