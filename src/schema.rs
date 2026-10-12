use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Schema {
  Boolean,
  Bytes,
  Enum { variants: Vec<Variant> },
  SignedInteger,
  String,
  UnsignedInteger,
}

impl Schema {
  pub fn register(self, name: &str, schemas: &mut BTreeMap<String, Schema>) {
    if let Some(existing) = schemas.get(name) {
      assert!(
        *existing == self,
        "conflicting definitions for schema `{name}`"
      );
    } else {
      schemas.insert(name.into(), self);
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Variant {
  pub n: u64,
  pub name: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn register() {
    let mut schemas = BTreeMap::new();

    Schema::Boolean.register("Foo", &mut schemas);
    Schema::Boolean.register("Foo", &mut schemas);

    assert_eq!(schemas, BTreeMap::from([("Foo".into(), Schema::Boolean)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `Foo`")]
  fn register_conflict() {
    let mut schemas = BTreeMap::new();

    Schema::Boolean.register("Foo", &mut schemas);
    Schema::Bytes.register("Foo", &mut schemas);
  }
}
