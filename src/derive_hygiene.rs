//! Derives expanded where nothing of filepack's prelude is in scope.

/// Foo.
#[derive(
  Debug, PartialEq, filepack_derive::Decode, filepack_derive::Encode, filepack_derive::Json,
)]
struct Foo {
  /// Bar.
  #[n(1)]
  bar: u64,
  /// Baz.
  #[n(2)]
  baz: String,
}

#[test]
fn round_trip() {
  use crate::{Decode, Encode};

  let foo = Foo {
    bar: 1,
    baz: "baz".into(),
  };

  assert_eq!(Foo::decode_from_slice(&foo.encode_to_vec()).unwrap(), foo);
}

#[test]
fn schemas() {
  use crate::{Encode, json::Json};

  assert_eq!(Foo::deco_schema(&mut crate::deco::Registry::new()), "Foo");
  assert_eq!(Foo::json_schema(&mut crate::json::Registry::new()), "Foo");
}
