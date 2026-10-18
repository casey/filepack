use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Field {
  pub description: String,
  pub example: Option<String>,
  pub name: String,
  pub optional: bool,
  pub schema: String,
}

impl Field {
  pub fn description(&self) -> String {
    self.description.replace('\n', " ")
  }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Registry {
  #[serde(skip)]
  pending: BTreeSet<String>,
  schemas: BTreeMap<String, Schema>,
}

impl Registry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn reserve(&mut self, name: &str) -> bool {
    !self.pending.insert(name.into())
  }
}

impl Deref for Registry {
  type Target = BTreeMap<String, Schema>;

  fn deref(&self) -> &Self::Target {
    &self.schemas
  }
}

impl<'a> IntoIterator for &'a Registry {
  type IntoIter = std::collections::btree_map::Iter<'a, String, Schema>;
  type Item = (&'a String, &'a Schema);

  fn into_iter(self) -> Self::IntoIter {
    self.schemas.iter()
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Schema {
  Array {
    element: String,
  },
  Boolean,
  Bytes,
  Hex {
    tag: Tag,
  },
  Map {
    key: String,
    value: String,
  },
  Set {
    element: String,
  },
  String,
  StringEnum {
    values: Vec<Value>,
  },
  Struct {
    fields: Vec<Field>,
    strict: bool,
  },
  TaggedEnum {
    strict: bool,
    tag: String,
    variants: Vec<Variant>,
  },
  UnsignedInteger,
  UntaggedEnum {
    variants: Vec<String>,
  },
  Wrapper {
    inner: String,
  },
}

impl Schema {
  pub fn register(self, name: &str, registry: &mut Registry) -> String {
    if let Some(existing) = registry.schemas.get(name) {
      assert!(
        *existing == self,
        "conflicting definitions for schema `{name}`"
      );
    } else {
      registry.pending.remove(name);
      registry.schemas.insert(name.into(), self);
    }

    name.into()
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Value {
  pub description: String,
  pub value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Variant {
  pub description: String,
  pub fields: Vec<Field>,
  pub name: String,
}

pub trait Io {
  fn io_schema(registry: &mut Registry) -> Option<String>;
}

impl<T: Json> Io for T {
  fn io_schema(registry: &mut Registry) -> Option<String> {
    Some(T::json_schema(registry))
  }
}

impl Io for () {
  fn io_schema(_registry: &mut Registry) -> Option<String> {
    None
  }
}

pub trait Json {
  fn json_schema(registry: &mut Registry) -> String;
}

impl Json for bool {
  fn json_schema(registry: &mut Registry) -> String {
    Schema::Boolean.register("Boolean", registry)
  }
}

impl<K: Json, V: Json> Json for BTreeMap<K, V> {
  fn json_schema(registry: &mut Registry) -> String {
    let key = K::json_schema(registry);
    let value = V::json_schema(registry);
    let name = format!("Map<{key}, {value}>");
    Schema::Map { key, value }.register(&name, registry)
  }
}

impl<T: Json> Json for BTreeSet<T> {
  fn json_schema(registry: &mut Registry) -> String {
    let element = T::json_schema(registry);
    let name = format!("Set<{element}>");
    Schema::Set { element }.register(&name, registry)
  }
}

impl Json for String {
  fn json_schema(registry: &mut Registry) -> String {
    Schema::String.register("String", registry)
  }
}

impl<T: Json> Json for Vec<T> {
  fn json_schema(registry: &mut Registry) -> String {
    let element = T::json_schema(registry);
    let name = format!("Array<{element}>");
    Schema::Array { element }.register(&name, registry)
  }
}

impl Json for u64 {
  fn json_schema(registry: &mut Registry) -> String {
    Schema::UnsignedInteger.register("UnsignedInteger", registry)
  }
}

impl Json for Utf8PathBuf {
  fn json_schema(registry: &mut Registry) -> String {
    Schema::Wrapper {
      inner: String::json_schema(registry),
    }
    .register("Utf8PathBuf", registry)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn register() {
    let mut schemas = Registry::new();

    assert_eq!(Schema::String.register("Foo", &mut schemas), "Foo");
    assert_eq!(Schema::String.register("Foo", &mut schemas), "Foo");

    assert_eq!(*schemas, BTreeMap::from([("Foo".into(), Schema::String)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `Foo`")]
  fn register_conflict() {
    let mut schemas = Registry::new();

    Schema::String.register("Foo", &mut schemas);
    Schema::Array {
      element: "Foo".into(),
    }
    .register("Foo", &mut schemas);
  }
}
