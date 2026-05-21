use {
  super::*,
  crate::server::DatabaseMetadata,
  redb::{Key, TypeName, Value},
};

impl Key for DatabaseMetadata {
  fn compare(a: &[u8], b: &[u8]) -> Ordering {
    u64::compare(a, b)
  }
}

impl Value for DatabaseMetadata {
  type AsBytes<'a>
    = <u64 as Value>::AsBytes<'a>
  where
    Self: 'a;

  type SelfType<'a>
    = Self
  where
    Self: 'a;

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    u64::as_bytes(&(*value as u64))
  }

  fn fixed_width() -> Option<usize> {
    u64::fixed_width()
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self
  where
    Self: 'a,
  {
    Self::from_repr(u64::from_bytes(data)).unwrap()
  }

  fn type_name() -> TypeName {
    TypeName::new("filepack-metadata-key")
  }
}

impl Key for Hash {
  fn compare(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)
  }
}

impl Value for Hash {
  type AsBytes<'a>
    = &'a [u8; Self::LEN]
  where
    Self: 'a;

  type SelfType<'a>
    = Hash
  where
    Self: 'a;

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'b,
  {
    value.as_bytes()
  }

  fn fixed_width() -> Option<usize> {
    Some(Self::LEN)
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    <[u8; Self::LEN]>::try_from(data).unwrap().into()
  }

  fn type_name() -> TypeName {
    TypeName::new("filepack-hash")
  }
}
