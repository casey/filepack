use super::*;

pub(crate) use self::{
  audio::Audio, image::Image, media::Media, metadata::Metadata, package_metadata::PackageMetadata,
  video::Video,
};

mod audio;
mod image;
mod media;
mod metadata;
mod package_metadata;
mod video;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Field {
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
  Enum {
    strict: bool,
    tag: String,
    variants: Vec<Variant>,
  },
  String,
  Struct {
    fields: Vec<Field>,
    strict: bool,
  },
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
  pub fields: Vec<Field>,
  pub name: String,
}

pub trait Yaml {
  fn yaml_schema(registry: &mut Registry) -> String;
}

impl Yaml for String {
  fn yaml_schema(registry: &mut Registry) -> String {
    Schema::String.register("String", registry)
  }
}

impl<T: Yaml> Yaml for Vec<T> {
  fn yaml_schema(registry: &mut Registry) -> String {
    let element = T::yaml_schema(registry);
    let name = format!("Array<{element}>");
    Schema::Array { element }.register(&name, registry)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn register() {
    let mut schemas = BTreeMap::new();

    assert_eq!(Schema::String.register("Foo", &mut schemas), "Foo");
    assert_eq!(Schema::String.register("Foo", &mut schemas), "Foo");

    assert_eq!(schemas, BTreeMap::from([("Foo".into(), Schema::String)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `Foo`")]
  fn register_conflict() {
    let mut schemas = BTreeMap::new();

    Schema::String.register("Foo", &mut schemas);
    Schema::Array {
      element: "Foo".into(),
    }
    .register("Foo", &mut schemas);
  }
}
