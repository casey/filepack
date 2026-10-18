use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Example {
  pub variants: Vec<Variant>,
  pub yaml: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Variant {
  pub field: String,
  pub name: String,
  pub yaml: String,
}

impl Example {
  pub(crate) fn new(registry: &json::Registry, name: &str) -> Self {
    let json::Schema::Struct { fields, .. } = &registry[name] else {
      panic!("schema `{name}` is not a struct");
    };

    let mut writer = Writer::new(registry);

    let mut example_variants = Vec::new();

    for field in fields {
      if let json::Schema::TaggedEnum { variants, .. } = &registry[&field.schema] {
        writer.blank();
        writer.placeholder(field);

        for variant in variants {
          let mut writer = Writer::new(registry);

          writer.variant(field, &variant.name);

          example_variants.push(Variant {
            field: field.name.clone(),
            name: variant.name.clone(),
            yaml: writer.finish(),
          });
        }
      } else {
        writer.blank();
        writer.field(field);
      }
    }

    Self {
      variants: example_variants,
      yaml: writer.finish(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn metadata() {
    let mut registry = json::Registry::new();

    Metadata::json_schema(&mut registry);

    let example = Example::new(&registry, "Metadata");

    assert!(example.yaml.contains("\nmedia: ...\n"));

    serde_yaml::from_str::<Metadata>(&example.yaml.replace(": ...", ":")).unwrap();

    assert_eq!(
      example
        .variants
        .iter()
        .map(|variant| (variant.field.as_str(), variant.name.as_str()))
        .collect::<Vec<(&str, &str)>>(),
      [
        ("media", "audio"),
        ("media", "image"),
        ("media", "video"),
        ("media", "web"),
      ],
    );

    for variant in &example.variants {
      serde_yaml::from_str::<Metadata>(&variant.yaml).unwrap();
    }
  }
}
