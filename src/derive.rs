use super::*;

#[test]
fn all_optional_all_none() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(1)]
    bar: Option<u64>,
    #[n(2)]
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
    #[n(1)]
    bar: Option<u64>,
    #[n(2)]
    baz: Option<String>,
  }

  assert_deco(
    Foo {
      bar: Some(1),
      baz: Some("foo".into()),
    },
    "8701010283666f6f",
  );
}

#[test]
fn all_optional_mixed() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(1)]
    bar: Option<u64>,
    #[n(2)]
    baz: Option<String>,
  }

  assert_deco(
    Foo {
      bar: Some(1),
      baz: None,
    },
    "820101",
  );

  assert_deco(
    Foo {
      bar: None,
      baz: Some("foo".into()),
    },
    "850283666f6f",
  );
}

#[test]
fn all_required() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(1)]
    bar: u64,
    #[n(2)]
    baz: String,
  }

  assert_deco(
    Foo {
      bar: 42,
      baz: "foo".into(),
    },
    "87012a0283666f6f",
  );
}

#[test]
fn borrowed_field() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  struct Foo<'a> {
    #[n(1)]
    bar: &'a [u8],
  }

  let buffer = Foo { bar: b"bar" }.encode_to_vec();

  assert_eq!(
    Foo::decode_from_slice(&buffer).unwrap(),
    Foo { bar: b"bar" },
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
      name: "foo",
      ref source,
    } if source.to_string() == "bar error",
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
fn enum_added_optional_field() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Bar {
    #[n(0)]
    Bar {
      #[n(1)]
      foo: Option<u64>,
    },
  }

  assert_deco(Foo::Bar, "00");
  assert_deco(Bar::Bar { foo: None }, "00");
  assert_deco(Bar::Bar { foo: Some(1) }, "8400820101");

  let bytes = Bar::Bar { foo: Some(1) }.encode_to_vec();
  assert_eq!(Foo::decode_from_slice(&bytes).unwrap(), Foo::Bar);
  assert_matches!(
    Foo::decode_from_slice_with_options(DecodeOptions::strict(), &bytes),
    Err(DecodeError::UnknownField { key: 1 }),
  );

  assert_eq!(
    Bar::decode_from_slice(&[0x84, 0, 0x82, 2, 2]).unwrap(),
    Bar::Bar { foo: None },
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
      #[n(1)]
      baz: u64,
    },
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x82, 0x05, 0x80]),
    Err(DecodeError::InvalidDiscriminant {
      discriminant: 5,
      name: "foo",
    }),
  );
}

#[test]
fn enum_array_missing_field() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(1)]
      bar: u64,
    },
  }

  assert_eq!(
    Foo::decode_from_slice(&[0x00]).unwrap_err().to_string(),
    "missing field with key 1",
  );
}

#[test]
fn enum_array_unconsumed_elements() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(1)]
      bar: u64,
    },
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x85, 0x00, 0x82, 0x01, 0x05, 0x00]),
    Err(DecodeError::UnconsumedElements),
  );
}

#[test]
fn enum_empty_map() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
    #[n(1)]
    Baz {
      #[n(1)]
      foo: Option<u64>,
    },
  }

  for options in [DecodeOptions::default(), DecodeOptions::strict()] {
    for tag in [0, 1] {
      assert_matches!(
        Foo::decode_from_slice_with_options(options, &[0x82, tag, 0x80]),
        Err(DecodeError::EmptyVariantMap),
      );
    }
  }
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
        name: "foo",
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
      #[n(1)]
      baz: u64,
    },
  }

  assert_deco(Foo::Bar, "00");
  assert_deco(Foo::Baz { baz: 99 }, "8401820163");
}

#[test]
fn enum_named_field() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(1)]
      bar: u64,
      #[n(2)]
      baz: String,
    },
  }

  assert_deco(
    Foo::Bar {
      bar: 42,
      baz: "foo".into(),
    },
    "890087012a0283666f6f",
  );
}

#[test]
fn enum_named_field_optional() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(1)]
      bar: Option<u64>,
      #[n(2)]
      baz: u64,
    },
  }

  assert_deco(
    Foo::Bar {
      bar: Some(1),
      baz: 2,
    },
    "86008401010202",
  );

  assert_deco(Foo::Bar { bar: None, baz: 2 }, "8400820202");
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
    Foo::decode_from_slice(&[0x85, 0, 0x82, 1, 1, 0]),
    Err(DecodeError::UnconsumedElements),
  );
}

#[test]
fn enum_unknown_fields() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(1)]
      foo: u64,
      #[n(2)]
      bar: Option<u64>,
    },
  }

  let bytes = [0x86, 0, 0x84, 1, 1, 3, 3];

  assert_eq!(
    Foo::decode_from_slice(&bytes).unwrap(),
    Foo::Bar { foo: 1, bar: None },
  );
  assert_matches!(
    Foo::decode_from_slice_with_options(DecodeOptions::strict(), &bytes),
    Err(DecodeError::UnknownField { key: 3 }),
  );
}

#[test]
fn enum_unknown_variants() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Bar {
    #[n(0)]
    Baz {
      #[n(1)]
      foo: Option<Foo>,
      #[n(2)]
      bar: u64,
    },
  }

  let bytes = [0x86, 0, 0x84, 1, 1, 2, 2];

  assert_eq!(
    Bar::decode_from_slice(&bytes).unwrap(),
    Bar::Baz { foo: None, bar: 2 },
  );
  assert_matches!(
    Bar::decode_from_slice_with_options(DecodeOptions::strict(), &bytes),
    Err(DecodeError::InvalidDiscriminant {
      discriminant: 1,
      name: "foo",
    }),
  );
}

#[test]
fn magic() {
  #[derive(Debug, Decode, Encode, Magic, PartialEq)]
  #[deco(magic = MagicType::Archive)]
  struct Foo {
    #[n(1)]
    bar: u64,
  }

  #[track_caller]
  fn case(bytes: &[u8], expected: &str) {
    assert_eq!(
      Foo::decode_from_slice(bytes).unwrap_err().to_string(),
      expected,
    );
  }

  assert_deco(Foo { bar: 1 }, "8966696c657061636b008761726368697665820101");

  case(
    b"\x83bar\x80",
    "expected magic bytes `filepack\\x00` but found `bar`",
  );

  case(
    b"\x910123456789abcdefg\x80",
    "expected magic bytes `filepack\\x00` but found `0123456789abcdef…`",
  );

  case(b"\x89filepack\0\x83foo\x80", "failed to parse magic type");

  case(
    b"\x89filepack\0\x88metadata\x80",
    "expected magic type `archive` but found `metadata`",
  );

  case(b"\x89filepack\0\x87archive\x80", "missing field with key 1");
}

#[test]
fn mixed_required_and_optional() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(1)]
    bar: Option<u64>,
    #[n(2)]
    baz: String,
  }

  assert_deco(
    Foo {
      bar: Some(1),
      baz: "foo".into(),
    },
    "8701010283666f6f",
  );

  assert_deco(
    Foo {
      bar: None,
      baz: "foo".into(),
    },
    "850283666f6f",
  );
}

#[test]
fn single_field() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(1)]
    bar: u64,
  }

  assert_deco(Foo { bar: 99 }, "820163");
}

#[test]
fn strict() {
  #[derive(Debug, Decode, PartialEq)]
  #[deco(strict)]
  struct Foo {
    #[n(1)]
    foo: u64,
  }

  #[derive(Debug, Decode, PartialEq)]
  #[deco(strict)]
  enum Bar {
    #[n(0)]
    Baz {
      #[n(1)]
      foo: u64,
    },
  }

  let fields = with_unknown_field(BTreeMap::from([(1u64, 1u64)]));

  assert_matches!(
    Foo::decode_from_slice(&fields),
    Err(DecodeError::UnknownField { key: u64::MAX }),
  );

  assert_matches!(
    Bar::decode_from_slice(&Encoder::frame([vec![0], fields].concat())),
    Err(DecodeError::UnknownField { key: u64::MAX }),
  );
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
fn unknown_variants() {
  #[derive(Debug, Decode, Encode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  #[derive(Debug, Decode, Encode, PartialEq)]
  struct Bar {
    #[n(1)]
    bar: Option<Foo>,
    #[n(2)]
    foo: Option<u64>,
  }

  for bar in [None, Some(Foo::Bar)] {
    let bar = Bar { bar, foo: Some(2) };
    let bytes = bar.encode_to_vec();
    assert_eq!(Bar::decode_from_slice(&bytes).unwrap(), bar);
    assert_eq!(
      Bar::decode_from_slice_with_options(DecodeOptions::strict(), &bytes).unwrap(),
      bar,
    );
  }

  let bytes = BTreeMap::<u64, Vec<u8>>::from([(1, vec![1, 0xf8]), (2, vec![2])]).encode_to_vec();
  assert_eq!(
    Bar::decode_from_slice(&bytes).unwrap(),
    Bar {
      bar: None,
      foo: Some(2)
    },
  );
  assert_matches!(
    Bar::decode_from_slice_with_options(DecodeOptions::strict(), &bytes),
    Err(DecodeError::InvalidDiscriminant {
      discriminant: 1,
      name: "foo",
    }),
  );
}

#[test]
fn unknown_variants_errors() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar {
      #[n(1)]
      foo: bool,
    },
  }

  #[derive(Debug, Decode, PartialEq)]
  struct Bar {
    #[n(1)]
    foo: Option<Foo>,
  }

  for (payload, expected) in [
    (vec![0x80], "empty integer"),
    (vec![0, 0x82, 1, 2], "invalid boolean value 2"),
  ] {
    let bytes = BTreeMap::<u64, Vec<u8>>::from([(1, payload)]).encode_to_vec();
    assert_eq!(
      Bar::decode_from_slice(&bytes).unwrap_err().to_string(),
      expected
    );
  }

  assert_matches!(
    Bar::decode_from_slice(&[0x83, 1, 0x82, 1]),
    Err(DecodeError::Truncated),
  );
}

#[test]
fn unknown_variants_scope() {
  #[derive(Debug, Decode, PartialEq)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  #[derive(Debug, Decode, PartialEq)]
  struct Bar<T> {
    #[n(1)]
    foo: Option<T>,
  }

  #[derive(Debug, Decode, PartialEq)]
  struct Baz {
    #[n(1)]
    foo: Foo,
  }

  #[derive(Debug, Decode, PartialEq)]
  #[deco(strict)]
  struct Qux<T> {
    #[n(1)]
    foo: Option<T>,
  }

  #[derive(Debug, Decode, PartialEq)]
  #[deco(strict)]
  enum Quux {
    #[n(0)]
    Foo {
      #[n(1)]
      foo: Option<Foo>,
    },
  }

  #[track_caller]
  fn rejects<T: Debug + DecodeOwned>(value: impl Encode) {
    assert_matches!(
      T::decode_from_slice(&value.encode_to_vec()),
      Err(DecodeError::InvalidDiscriminant {
        discriminant: 1,
        name: "foo",
      }),
    );
  }

  let fields = BTreeMap::from([(1u64, vec![1u64])]);
  rejects::<Baz>(&fields);
  rejects::<Qux<Foo>>(&fields);
  rejects::<Bar<Qux<Foo>>>(BTreeMap::from([(1u64, &fields)]));
  rejects::<Bar<Vec<Foo>>>(BTreeMap::from([(1u64, vec![vec![1u64]])]));
  rejects::<Bar<Quux>>(BTreeMap::from([(
    1u64,
    [vec![0], fields.encode_to_vec()].concat(),
  )]));
  rejects::<Qux<Bar<Foo>>>(BTreeMap::from([(1u64, &fields)]));
}

#[test]
fn unknown_variants_validate() {
  #[derive(Debug, Decode, PartialEq)]
  #[deco(validate)]
  enum Foo {
    #[n(0)]
    Bar,
  }

  impl Validate for Foo {
    fn validate(&self) -> DecodeResult {
      Err(DecodeError::UnexpectedKey)
    }
  }

  #[derive(Debug, Decode, PartialEq)]
  #[deco(validate)]
  struct Bar {
    #[n(1)]
    foo: Option<Foo>,
  }

  impl Validate for Bar {
    fn validate(&self) -> DecodeResult {
      ensure!(self.foo.is_some(), decode_error::MissingElement);
      Ok(())
    }
  }

  assert_matches!(
    Bar::decode_from_slice(&with_unknown_field(BTreeMap::from([(1u64, vec![1u64])]))),
    Err(DecodeError::MissingElement),
  );
  assert_matches!(
    Bar::decode_from_slice(&BTreeMap::from([(1u64, vec![0u64])]).encode_to_vec()),
    Err(DecodeError::UnexpectedKey),
  );
}

#[test]
fn unsupported_version() {
  #[derive(Debug, Decode, PartialEq)]
  struct Foo {
    #[n(1)]
    foo: u64,
  }

  #[derive(Debug, Decode, PartialEq)]
  enum Bar {
    #[n(0)]
    Baz,
    #[n(1)]
    Qux {
      #[n(1)]
      foo: u64,
    },
  }

  #[track_caller]
  fn case<T: Debug + DecodeOwned>(bytes: &[u8], expected: &str) {
    for options in [DecodeOptions::default(), DecodeOptions::strict()] {
      assert_eq!(
        T::decode_from_slice_with_options(options, bytes)
          .unwrap_err()
          .to_string(),
        expected,
      );
    }
  }

  assert_matches!(
    Foo::decode_from_slice(&[0x84, 0, 1, 1, 1]),
    Err(DecodeError::UnsupportedVersion {
      name: "foo",
      version: 1,
    }),
  );

  case::<Foo>(&[0x84, 0, 1, 1, 1], "unsupported version 1 for foo");
  case::<Foo>(&[0x84, 0, 0, 1, 1], "unsupported version 0 for foo");
  case::<Bar>(&[0x84, 1, 0x82, 0, 1], "unsupported version 1 for bar qux");
  case::<Bar>(&[0x84, 0, 0x82, 0, 1], "unsupported version 1 for bar baz");
}

#[test]
fn validate() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  #[deco(transparent, validate)]
  struct Foo(String);

  impl Validate for Foo {
    fn validate(&self) -> DecodeResult {
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
      #[n(1)]
      baz: String,
    },
  }

  impl Validate for Foo {
    fn validate(&self) -> DecodeResult {
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

  assert_deco(Foo::Bar { baz: "foo".into() }, "8700850183666f6f");

  let fields = BTreeMap::from([(1u64, "bar"), (2, "baz")]).encode_to_vec();
  let bytes = Encoder::frame([vec![0], fields].concat());

  assert_matches!(
    Foo::decode_from_slice(&bytes),
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
    #[n(1)]
    bar: String,
  }

  impl Validate for Foo {
    fn validate(&self) -> DecodeResult {
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

  assert_deco(Foo { bar: "foo".into() }, "850183666f6f");

  assert_matches!(
    Foo::decode_from_slice(&Foo { bar: "bar".into() }.encode_to_vec()),
    Err(DecodeError::UnexpectedValue {
      actual,
      expected: "foo",
    }) if actual == "bar",
  );
}
