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
    registry.register("String", Schema::String)
  }
}

impl<T: Yaml> Yaml for Vec<T> {
  fn yaml_schema(registry: &mut Registry) -> String {
    let element = T::yaml_schema(registry);
    let name = format!("Array<{element}>");
    registry.register(&name, Schema::Array { element })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn register() {
    let mut schemas = Registry::new();

    assert_eq!(schemas.register("Foo", Schema::String), "Foo");
    assert_eq!(schemas.register("Foo", Schema::String), "Foo");

    assert_eq!(*schemas, BTreeMap::from([("Foo".into(), Schema::String)]));
  }

  #[test]
  #[should_panic(expected = "conflicting definitions for schema `Foo`")]
  fn register_conflict() {
    let mut schemas = Registry::new();

    schemas.register("Foo", Schema::String);
    schemas.register(
      "Foo",
      Schema::Array {
        element: "Foo".into(),
      },
    );
  }
}
