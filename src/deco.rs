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
  Boolean {
    description: String,
  },
  Bytes {
    length: Option<u64>,
  },
  Enum {
    description: String,
    variants: Vec<Variant>,
  },
  Map {
    key: String,
    value: String,
  },
  NonEmptyOrderedSet {
    element: String,
  },
  SignedInteger {
    description: String,
  },
  SortedSet {
    element: String,
  },
  String {
    description: String,
  },
  Struct {
    description: String,
    fields: Vec<Field>,
    magic: Option<String>,
    strict: bool,
  },
  UnsignedInteger {
    description: String,
  },
  Wrapper {
    description: String,
    inner: String,
  },
}

impl Schema {
  pub fn description(&self) -> Option<&str> {
    match self {
      Self::Array { .. }
      | Self::Bytes { .. }
      | Self::Map { .. }
      | Self::NonEmptyOrderedSet { .. }
      | Self::SortedSet { .. } => None,
      Self::Boolean { description }
      | Self::Enum { description, .. }
      | Self::SignedInteger { description }
      | Self::String { description }
      | Self::Struct { description, .. }
      | Self::UnsignedInteger { description }
      | Self::Wrapper { description, .. } => Some(description),
    }
  }

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
  fn description() {
    assert_eq!(
      Schema::Boolean {
        description: "Foo.".into()
      }
      .description(),
      Some("Foo."),
    );

    assert_eq!(Schema::Bytes { length: None }.description(), None);
  }

  #[test]
  fn register() {
    let mut schemas = BTreeMap::new();

    let schema = Schema::Boolean {
      description: "Foo.".into(),
    };

    assert_eq!(schema.clone().register("Foo", &mut schemas), "Foo");
    assert_eq!(schema.clone().register("Foo", &mut schemas), "Foo");

    assert_eq!(schemas, BTreeMap::from([("Foo".into(), schema)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `Foo`")]
  fn register_conflict() {
    let mut schemas = BTreeMap::new();

    Schema::Boolean {
      description: "Foo.".into(),
    }
    .register("Foo", &mut schemas);
    Schema::Bytes { length: None }.register("Foo", &mut schemas);
  }
}
