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
fn enum_integer_range() {
  #[derive(Clone, Copy, Debug, Decode, Encode, FromRepr, PartialEq)]
  #[repr(u8)]
  enum Foo {
    Bar = 0,
  }

  assert_matches!(
    Foo::decode_from_slice(&256u64.encode_to_vec()),
    Err(DecodeError::IntegerRange { .. }),
  );
}

#[test]
fn enum_invalid_discriminant() {
  #[derive(Clone, Copy, Debug, Decode, Encode, FromRepr, PartialEq)]
  #[repr(u8)]
  enum Foo {
    Bar = 0,
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x01]),
    Err(DecodeError::InvalidDiscriminant {
      discriminant: 1,
      name: "Foo",
    }),
  );
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
  #[derive(Clone, Copy, Debug, Decode, Encode, FromRepr, PartialEq)]
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
