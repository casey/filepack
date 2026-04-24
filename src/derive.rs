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

  assert_encoding(Foo {
    bar: None,
    baz: None,
  });
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

  assert_encoding(Foo {
    bar: Some(1),
    baz: Some("foo".into()),
  });
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

  assert_encoding(Foo {
    bar: Some(1),
    baz: None,
  });

  assert_encoding(Foo {
    bar: None,
    baz: Some("foo".into()),
  });
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

  assert_encoding(Foo {
    bar: 42,
    baz: "foo".into(),
  });
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

  assert_encoding(Foo {
    bar: Some(1),
    baz: "foo".into(),
  });

  assert_encoding(Foo {
    bar: None,
    baz: "foo".into(),
  });
}

#[test]
fn single_field() {
  #[derive(Debug, Encode, Decode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: u64,
  }

  assert_encoding(Foo { bar: 99 });
}
