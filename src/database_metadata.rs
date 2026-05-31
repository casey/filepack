use super::*;

#[derive(Copy, Clone, Debug, FromRepr)]
#[repr(u64)]
pub(crate) enum DatabaseMetadata {
  Schema = 0,
}

impl redb::Key for DatabaseMetadata {
  fn compare(a: &[u8], b: &[u8]) -> Ordering {
    u64::compare(a, b)
  }
}

impl redb::Value for DatabaseMetadata {
  type AsBytes<'a>
    = <u64 as redb::Value>::AsBytes<'a>
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

  fn type_name() -> redb::TypeName {
    redb::TypeName::new("filepack-metadata-key")
  }
}
