use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Schema {
  Boolean,
  Bytes,
  Enum { variants: Vec<Variant> },
  I64,
  String,
  U64,
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

    Schema::U64.register("foo", &mut schemas);
    Schema::U64.register("foo", &mut schemas);

    assert_eq!(schemas, BTreeMap::from([("foo".into(), Schema::U64)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `foo`")]
  fn register_conflict() {
    let mut schemas = BTreeMap::new();

    Schema::U64.register("foo", &mut schemas);
    Schema::I64.register("foo", &mut schemas);
  }
}
