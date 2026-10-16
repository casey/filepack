use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Field {
  pub n: u64,
  pub name: String,
  pub optional: bool,
  pub schema: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Registry(BTreeMap<String, Schema>);

impl Registry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn register(&mut self, name: &str, schema: Schema) -> String {
    if let Some(existing) = self.0.get(name) {
      assert!(
        *existing == schema,
        "conflicting definitions for schema `{name}`"
      );
    } else {
      self.0.insert(name.into(), schema);
    }

    name.into()
  }

  pub(crate) fn remove(&mut self, name: &str) -> Option<Schema> {
    self.0.remove(name)
  }
}

impl Deref for Registry {
  type Target = BTreeMap<String, Schema>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

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
  SignedInteger,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Variant {
  pub fields: Vec<Field>,
  pub n: u64,
  pub name: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn register() {
    let mut schemas = Registry::new();

    assert_eq!(schemas.register("Foo", Schema::Boolean), "Foo");
    assert_eq!(schemas.register("Foo", Schema::Boolean), "Foo");

    assert_eq!(*schemas, BTreeMap::from([("Foo".into(), Schema::Boolean)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `Foo`")]
  fn register_conflict() {
    let mut schemas = Registry::new();

    schemas.register("Foo", Schema::Boolean);
    schemas.register("Foo", Schema::Bytes { length: None });
  }
}
