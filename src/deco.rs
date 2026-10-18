use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Field {
  pub description: String,
  pub n: u64,
  pub name: String,
  pub optional: bool,
  pub schema: String,
}

pub type Registry = BTreeMap<String, Schema>;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Schema {
  Array {
    element: String,
  },
  Boolean,
  Bytes {
    length: Option<u64>,
  },
  Enum {
    variants: Vec<Variant>,
  },
  Map {
    key: String,
    value: String,
  },
  NonEmptyOrderedSet {
    element: String,
  },
  SignedInteger,
  SortedSet {
    element: String,
  },
  String,
  Struct {
    fields: Vec<Field>,
    magic: Option<String>,
    strict: bool,
  },
  UnsignedInteger,
  Wrapper {
    inner: String,
  },
}

impl Schema {
  pub fn register(self, name: &str, registry: &mut Registry) -> String {
    if let Some(existing) = registry.get(name) {
      assert!(
        *existing == self,
        "conflicting definitions for schema `{name}`"
      );
    } else {
      registry.insert(name.into(), self);
    }

    name.into()
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Variant {
  pub description: String,
  pub fields: Vec<Field>,
  pub n: u64,
  pub name: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn register() {
    let mut schemas = BTreeMap::new();

    assert_eq!(Schema::Boolean.register("Foo", &mut schemas), "Foo");
    assert_eq!(Schema::Boolean.register("Foo", &mut schemas), "Foo");

    assert_eq!(schemas, BTreeMap::from([("Foo".into(), Schema::Boolean)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `Foo`")]
  fn register_conflict() {
    let mut schemas = BTreeMap::new();

    Schema::Boolean.register("Foo", &mut schemas);
    Schema::Bytes { length: None }.register("Foo", &mut schemas);
  }
}
