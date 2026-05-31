use super::*;

#[derive(Display)]
pub(crate) enum TypeName {
  DatabaseMetadata,
  Fingerprint,
  Hash,
}

impl From<TypeName> for redb::TypeName {
  fn from(type_name: TypeName) -> redb::TypeName {
    redb::TypeName::new(&format!("filepack::{type_name}"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn type_name() {
    assert_eq!(
      redb::TypeName::from(TypeName::DatabaseMetadata).name(),
      "filepack::DatabaseMetadata",
    );
  }
}
