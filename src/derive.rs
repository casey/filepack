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
    &[0xa0],
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
    &[0xa2, 0x00, 0x01, 0x01, 0x63, 0x66, 0x6f, 0x6f],
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
    &[0xa1, 0x00, 0x01],
  );

  assert_cbor(
    Foo {
      bar: None,
      baz: Some("foo".into()),
    },
    &[0xa1, 0x01, 0x63, 0x66, 0x6f, 0x6f],
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
    &[0xa2, 0x00, 0x18, 0x2a, 0x01, 0x63, 0x66, 0x6f, 0x6f],
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
fn enum_invalid_discriminant() {
  #[derive(Debug, Decode, FromRepr)]
  #[repr(u8)]
  enum Foo {
    Bar = 0,
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
fn enum_round_trip() {
  #[derive(Clone, Copy, Debug, Decode, Encode, FromRepr, PartialEq)]
  #[repr(u8)]
  enum Foo {
    Bar = 0,
    Baz = 1,
  }

  assert_cbor(Foo::Bar, &[0x00]);
  assert_cbor(Foo::Baz, &[0x01]);
}

#[test]
fn enum_unexpected_type() {
  #[derive(Debug, Decode, FromRepr)]
  #[repr(u8)]
  enum Foo {
    Bar = 0,
  }

  assert_matches!(
    Foo::decode_from_slice(&"foo".encode_to_vec()),
    Err(DecodeError::UnexpectedType {
      expected: MajorType::UnsignedInteger,
      actual: MajorType::Text,
    }),
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
    &[0xa2, 0x00, 0x01, 0x01, 0x63, 0x66, 0x6f, 0x6f],
  );

  assert_cbor(
    Foo {
      bar: None,
      baz: "foo".into(),
    },
    &[0xa1, 0x01, 0x63, 0x66, 0x6f, 0x6f],
  );
}

#[test]
fn single_field() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: u64,
  }

  assert_cbor(Foo { bar: 99 }, &[0xa1, 0x00, 0x18, 0x63]);
}

#[test]
fn transparent_named() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[transparent]
  struct Foo {
    bar: String,
  }

  assert_cbor(Foo { bar: "foo".into() }, &[0x63, 0x66, 0x6f, 0x6f]);
  assert_cbor(Foo { bar: "foo".into() }, &"foo".encode_to_vec());
}

#[test]
fn transparent_newtype() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[transparent]
  struct Foo(u64);

  assert_cbor(Foo(99), &[0x18, 0x63]);
  assert_cbor(Foo(99), &99u64.encode_to_vec());
}
