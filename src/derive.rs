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

  assert_deco(
    Foo {
      bar: None,
      baz: None,
    },
    "80",
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

  assert_deco(
    Foo {
      bar: Some(1),
      baz: Some("foo".into()),
    },
    "8700010183666f6f",
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

  assert_deco(
    Foo {
      bar: Some(1),
      baz: None,
    },
    "820001",
  );

  assert_deco(
    Foo {
      bar: None,
      baz: Some("foo".into()),
    },
    "850183666f6f",
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

  assert_deco(
    Foo {
      bar: 42,
      baz: "foo".into(),
    },
    "87002a0183666f6f",
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
    Foo::decode_from_slice(&[0x83, 0x66, 0x6f, 0x6f]).unwrap(),
    Foo("foo".to_string()),
  );

  let err = Foo::decode_from_slice(&[0x83, 0x62, 0x61, 0x72]).unwrap_err();

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
    #[deco(decode_with = decode_offset)]
    #[n(0)]
    bar: Option<u64>,
  }

  assert_eq!(
    Foo::decode_from_slice(&[0x82, 0x00, 0x63]).unwrap(),
    Foo { bar: Some(100) },
  );

  assert_eq!(Foo::decode_from_slice(&[0x80]).unwrap(), Foo { bar: None });
}

#[test]
fn decode_with_required() {
  fn decode_offset(decoder: &mut Decoder) -> Result<u64, DecodeError> {
    Ok(decoder.integer()? + 1)
  }

  #[derive(Debug, Decode, PartialEq)]
  struct Foo {
    #[deco(decode_with = decode_offset)]
    #[n(0)]
    bar: u64,
  }

  assert_eq!(
    Foo::decode_from_slice(&[0x82, 0x00, 0x63]).unwrap(),
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

  assert_eq!(Foo.encode_to_vec(), [0x83, 0x66, 0x6f, 0x6f]);
}

#[test]
fn encode_with_optional() {
  struct Foreign(u64);

  fn encode_foreign(value: &Foreign, encoder: &mut Encoder) {
    (value.0 + 1).encode(encoder);
  }

  #[derive(Encode)]
  struct Foo {
    #[deco(encode_with = encode_foreign)]
    #[n(0)]
    bar: Option<Foreign>,
  }

  assert_eq!(
    Foo {
      bar: Some(Foreign(99)),
    }
    .encode_to_vec(),
    [0x82, 0x00, 0x64],
  );

  assert_eq!(Foo { bar: None }.encode_to_vec(), [0x80]);
}

#[test]
fn encode_with_required() {
  struct Foreign(u64);

  fn encode_foreign(value: &Foreign, encoder: &mut Encoder) {
    (value.0 + 1).encode(encoder);
  }

  #[derive(Encode)]
  struct Foo {
    #[deco(encode_with = encode_foreign)]
    #[n(0)]
    bar: Foreign,
  }

  assert_eq!(Foo { bar: Foreign(99) }.encode_to_vec(), [0x82, 0x00, 0x64],);
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
    Foo::decode_from_slice(&[0x82, 0x05, 0x80]),
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
    Foo::decode_from_slice(&[0x00]),
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
    Foo::decode_from_slice(&[0x85, 0x00, 0x82, 0x00, 0x05, 0x00]),
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
  case(&vec![256u64].encode_to_vec(), 256);
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

  assert_deco(Foo::Bar, "00");
  assert_deco(Foo::Baz { baz: 99 }, "8401820063");
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

  assert_deco(
    Foo::Bar {
      bar: 42,
      baz: "foo".into(),
    },
    "890087002a0183666f6f",
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

  assert_deco(
    Foo::Bar {
      bar: Some(1),
      baz: 2,
    },
    "86008400010102",
  );

  assert_deco(Foo::Bar { bar: None, baz: 2 }, "8400820102");
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

  assert_deco(Foo::Bar, "00");
  assert_deco(Foo::Baz, "01");
}

#[test]
fn enum_unconsumed_unit_payload() {
  #[derive(Debug, Decode)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x82, 0x00, 0x00]),
    Err(DecodeError::UnconsumedElements),
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
      #[deco(encode_with = encode_foreign)]
      #[n(0)]
      bar: Foreign,
    },
  }

  assert_eq!(
    Foo::Bar { bar: Foreign(99) }.encode_to_vec(),
    [0x84, 0x00, 0x82, 0x00, 0x64],
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

  assert_deco(
    Foo {
      bar: Some(1),
      baz: "foo".into(),
    },
    "8700010183666f6f",
  );

  assert_deco(
    Foo {
      bar: None,
      baz: "foo".into(),
    },
    "850183666f6f",
  );
}

#[test]
fn single_field() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: u64,
  }

  assert_deco(Foo { bar: 99 }, "820063");
}

#[test]
fn transparent_named() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[deco(transparent)]
  struct Foo {
    bar: String,
  }

  assert_deco(Foo { bar: "foo".into() }, "83666f6f");
  assert_deco_eq(Foo { bar: "foo".into() }, "foo");
}

#[test]
fn transparent_newtype() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[deco(transparent)]
  struct Foo(u64);

  assert_deco(Foo(99), "63");
  assert_deco_eq(Foo(99), 99u64);
}

#[test]
fn validate() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[deco(transparent, validate)]
  struct Foo(String);

  impl Validate for Foo {
    fn validate(&self) -> Result<(), DecodeError> {
      ensure!(
        self.0 == "foo",
        decode_error::UnexpectedValue {
          actual: self.0.clone(),
          expected: "foo",
        }
      );
      Ok(())
    }
  }

  assert_deco(Foo("foo".into()), "83666f6f");

  assert_matches!(
    Foo::decode_from_slice(&"bar".encode_to_vec()),
    Err(DecodeError::UnexpectedValue {
      actual,
      expected: "foo",
    }) if actual == "bar",
  );
}

#[test]
fn validate_enum() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[deco(validate)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(0)]
      baz: String,
    },
  }

  impl Validate for Foo {
    fn validate(&self) -> Result<(), DecodeError> {
      let Self::Bar { baz } = self;
      ensure!(
        baz == "foo",
        decode_error::UnexpectedValue {
          actual: baz.clone(),
          expected: "foo",
        }
      );
      Ok(())
    }
  }

  assert_deco(Foo::Bar { baz: "foo".into() }, "8700850083666f6f");

  assert_matches!(
    Foo::decode_from_slice(&Foo::Bar { baz: "bar".into() }.encode_to_vec()),
    Err(DecodeError::UnexpectedValue {
      actual,
      expected: "foo",
    }) if actual == "bar",
  );
}

#[test]
fn validate_struct() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[deco(validate)]
  struct Foo {
    #[n(0)]
    bar: String,
  }

  impl Validate for Foo {
    fn validate(&self) -> Result<(), DecodeError> {
      ensure!(
        self.bar == "foo",
        decode_error::UnexpectedValue {
          actual: self.bar.clone(),
          expected: "foo",
        }
      );
      Ok(())
    }
  }

  assert_deco(Foo { bar: "foo".into() }, "850083666f6f");

  assert_matches!(
    Foo::decode_from_slice(&Foo { bar: "bar".into() }.encode_to_vec()),
    Err(DecodeError::UnexpectedValue {
      actual,
      expected: "foo",
    }) if actual == "bar",
  );
}
