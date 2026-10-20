use {super::*, clap::CommandFactory};

pub use self::{
  command::Command, content_type::ContentType, endpoint::Endpoint, hex::Hex, lint_group::LintGroup,
  media_format::MediaFormat, parameter::Parameter, path_parameter::PathParameter,
  query_parameter::QueryParameter, request::Request, response::Response, route::Route,
};

mod command;
mod content_type;
mod endpoint;
mod hex;
mod lint_group;
mod media_format;
mod parameter;
mod path_parameter;
mod query_parameter;
mod request;
mod response;
mod route;

#[derive(Debug, Deserialize, Serialize)]
pub struct Reference {
  pub commands: Command,
  pub content_types: Vec<ContentType>,
  pub deco: deco::Registry,
  pub hex: BTreeMap<Tag, Hex>,
  pub json: json::Registry,
  pub junk_names: Vec<String>,
  pub languages: BTreeMap<Language, String>,
  pub lint_groups: BTreeMap<crate::LintGroup, LintGroup>,
  pub lints: BTreeMap<Lint, String>,
  pub media_formats: BTreeMap<MediaType, Vec<MediaFormat>>,
  pub metadata: yaml::Example,
  pub routes: BTreeMap<String, Route>,
  pub unstable_features: BTreeMap<UnstableFeature, String>,
  pub windows_reserved_characters: Vec<char>,
  pub windows_reserved_names: Vec<String>,
  pub yaml: json::Registry,
}

impl Reference {
  pub fn new() -> Self {
    fn descriptions<T: Json>() -> BTreeMap<String, String> {
      T::schema()
        .values()
        .iter()
        .map(|value| (value.value.clone(), value.description.clone()))
        .collect()
    }

    fn media_formats<F: Format + Json + Serialize>() -> Vec<MediaFormat> {
      let descriptions = descriptions::<F>();

      let mut formats = BTreeMap::new();

      for extension in F::EXTENSIONS {
        let format = F::from_extension(extension).unwrap();

        let name = format.to_string();

        let value = serde_json::to_value(format).unwrap();

        let description = descriptions[value.as_str().unwrap()].clone();

        assert!(
          formats
            .insert(name.clone(), (description, (*extension).to_string()))
            .is_none(),
          "format `{name}` has multiple extensions",
        );
      }

      formats
        .into_iter()
        .map(|(name, (description, extension))| MediaFormat {
          description,
          extension,
          name,
        })
        .collect()
    }

    fn tag<T: crate::Hex>(
      registry: &mut deco::Registry,
      hex: &mut BTreeMap<Tag, Hex>,
      tags: &BTreeMap<String, String>,
    ) {
      hex.insert(
        T::TAG,
        Hex {
          description: tags[T::TAG.prefix()].clone(),
          example: T::EXAMPLE.into(),
          example_description: T::EXAMPLE_DESCRIPTION.into(),
          name: T::TAG.name().into(),
          schema: T::deco_schema(registry),
        },
      );
    }

    let mut command = Arguments::command().term_width(80);

    command.build();

    let mut deco = BTreeMap::new();

    Archive::deco_schema(&mut deco);
    Directory::deco_schema(&mut deco);
    Envelope::<Claims>::deco_schema(&mut deco);
    Envelope::<Statement>::deco_schema(&mut deco);
    Metadata::deco_schema(&mut deco);
    RevisionObject::deco_schema(&mut deco);

    let tags = descriptions::<Tag>();

    let mut hex = BTreeMap::new();

    tag::<Attestation>(&mut deco, &mut hex, &tags);
    tag::<DisplayPrivateKey>(&mut deco, &mut hex, &tags);
    tag::<Fingerprint>(&mut deco, &mut hex, &tags);
    tag::<Hash>(&mut deco, &mut hex, &tags);
    tag::<PublicKey>(&mut deco, &mut hex, &tags);
    tag::<Revision>(&mut deco, &mut hex, &tags);
    tag::<Token>(&mut deco, &mut hex, &tags);

    let mut json = json::Registry::new();

    let commands = Command::new(&mut command, None, &mut json);

    let lint_groups = descriptions::<crate::LintGroup>();

    let lints = descriptions::<Lint>();

    let unstable_features = descriptions::<UnstableFeature>();

    let mut yaml = json::Registry::new();

    yaml::Metadata::json_schema(&mut yaml);

    let metadata = yaml::Example::new(&yaml, "Metadata");

    let mut routes = BTreeMap::<String, Route>::new();

    for route in subcommand::serve::route::Route::all(&mut deco) {
      routes
        .entry(route.path.into())
        .or_insert_with(|| Route {
          api: route.path.starts_with("/api/"),
          methods: BTreeMap::new(),
          parameters: route.parameters.clone(),
          wildcard: route.path.contains("{*"),
        })
        .methods
        .insert(route.method.to_string(), route.endpoint);
    }

    Self {
      commands,
      content_types: ResourceType::iter()
        .filter(|ty| !ty.extensions().is_empty())
        .map(|ty| ContentType {
          content_type: ty.content_type().to_string(),
          extensions: ty
            .extensions()
            .iter()
            .map(|extension| (*extension).into())
            .collect(),
          sandbox: ty.sandbox(),
        })
        .collect(),
      deco,
      hex,
      json,
      junk_names: RelativePath::JUNK_NAMES
        .iter()
        .map(|name| (*name).into())
        .collect(),
      languages: language::ALL
        .iter()
        .map(|language| (*language, language.name().into()))
        .collect(),
      lint_groups: crate::LintGroup::iter()
        .map(|group| {
          (
            group,
            LintGroup {
              description: lint_groups[&group.to_string()].clone(),
              lints: group.lints(),
            },
          )
        })
        .collect(),
      lints: Lint::iter()
        .map(|lint| (lint, lints[&lint.to_string()].clone()))
        .collect(),
      media_formats: [
        (MediaType::Audio, media_formats::<AudioFormat>()),
        (MediaType::Image, media_formats::<ImageFormat>()),
        (MediaType::Video, media_formats::<VideoFormat>()),
      ]
      .into(),
      metadata,
      routes,
      unstable_features: UnstableFeature::iter()
        .map(|feature| (feature, unstable_features[feature.name()].clone()))
        .collect(),
      windows_reserved_characters: RelativePath::WINDOWS_RESERVED_CHARACTERS.into(),
      windows_reserved_names: RelativePath::WINDOWS_RESERVED_NAMES
        .iter()
        .map(|name| (*name).into())
        .collect(),
      yaml,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn content_types() {
    assert_eq!(
      Reference::new().content_types[..2],
      [
        ContentType {
          content_type: "audio/flac".into(),
          extensions: vec!["flac".into()],
          sandbox: false,
        },
        ContentType {
          content_type: "image/jpeg".into(),
          extensions: vec!["jpeg".into(), "jpg".into()],
          sandbox: true,
        },
      ],
    );
  }

  #[test]
  fn descriptions() {
    fn collect(value: &serde_json::Value, variant: bool, descriptions: &mut Vec<(String, bool)>) {
      match value {
        serde_json::Value::Array(array) => {
          for value in array {
            collect(value, variant, descriptions);
          }
        }
        serde_json::Value::Object(object) => {
          for (key, value) in object {
            if key == "description"
              && let serde_json::Value::String(description) = value
            {
              descriptions.push((description.clone(), variant));
            } else {
              collect(value, key == "values" || key == "variants", descriptions);
            }
          }
        }
        serde_json::Value::Bool(_)
        | serde_json::Value::Null
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
      }
    }

    let reference = serde_json::to_value(Reference::new()).unwrap();

    let mut descriptions = Vec::new();

    collect(&reference, false, &mut descriptions);

    for lint in reference["lints"].as_object().unwrap().values() {
      descriptions.push((lint.as_str().unwrap().into(), false));
    }

    for feature in reference["unstable_features"].as_object().unwrap().values() {
      descriptions.push((feature.as_str().unwrap().into(), false));
    }

    for hex in reference["hex"].as_object().unwrap().values() {
      descriptions.push((hex["example_description"].as_str().unwrap().into(), false));
    }

    assert!(descriptions.len() > 100);

    let mut errors = Vec::new();

    for (description, variant) in descriptions {
      if !description.ends_with('.') {
        errors.push(format!("`{description}` does not end with a period"));
      }

      if !description.starts_with(|c: char| c.is_ascii_uppercase() || c == '`' || c == '[') {
        errors.push(format!(
          "`{description}` does not start with an uppercase letter, backtick, or link"
        ));
      }

      if !variant
        && ["A ", "An ", "The "]
          .iter()
          .any(|article| description.starts_with(article))
      {
        errors.push(format!("`{description}` starts with an article"));
      }

      for line in description.lines() {
        if line.is_empty() || line != line.trim() || line.contains("  ") {
          errors.push(format!("`{description}` has irregular whitespace"));
          break;
        }
      }
    }

    assert_eq!(errors, Vec::<String>::new());
  }

  #[test]
  fn examples() {
    fn fields(registry: &json::Registry) -> Vec<(&str, &json::Field)> {
      let mut fields = Vec::new();

      for (name, schema) in registry {
        match schema {
          json::Schema::Struct { fields: inner, .. } => {
            fields.extend(inner.iter().map(|field| (name.as_str(), field)));
          }
          json::Schema::TaggedEnum { variants, .. } => {
            for variant in variants {
              fields.extend(variant.fields.iter().map(|field| (name.as_str(), field)));
            }
          }
          json::Schema::Array { .. }
          | json::Schema::Boolean { .. }
          | json::Schema::Bytes { .. }
          | json::Schema::Hex { .. }
          | json::Schema::Map { .. }
          | json::Schema::Set { .. }
          | json::Schema::String { .. }
          | json::Schema::StringEnum { .. }
          | json::Schema::UnsignedInteger { .. }
          | json::Schema::UntaggedEnum { .. }
          | json::Schema::Wrapper { .. } => {}
        }
      }

      fields
    }

    let reference = Reference::new();

    let mut errors = Vec::new();

    for (name, field) in fields(&reference.json) {
      if field.example.is_some() {
        errors.push(format!("json field `{name}.{}` has an example", field.name));
      }
    }

    for (name, field) in fields(&reference.yaml) {
      let scalar = !matches!(
        reference.yaml[&field.schema],
        json::Schema::Array { .. }
          | json::Schema::Map { .. }
          | json::Schema::Set { .. }
          | json::Schema::Struct { .. }
          | json::Schema::TaggedEnum { .. }
          | json::Schema::UntaggedEnum { .. }
      );

      if scalar && field.example.is_none() {
        errors.push(format!("yaml field `{name}.{}` has no example", field.name));
      }

      if !scalar && field.example.is_some() {
        errors.push(format!("yaml field `{name}.{}` has an example", field.name));
      }
    }

    assert_eq!(errors, Vec::<String>::new());
  }

  #[test]
  fn hex() {
    let hex = Reference::new().hex;

    assert_eq!(
      hex.keys().copied().collect::<Vec<Tag>>(),
      Tag::iter().collect::<Vec<Tag>>(),
    );

    assert_eq!(
      hex[&Tag::Token].description,
      "Signed bearer token for server authentication.",
    );

    assert_eq!(
      hex[&Tag::Hash].example_description,
      "Hash of the empty string.",
    );

    for (tag, value) in hex {
      assert!(
        value.example.starts_with(&format!("{}1", tag.prefix())),
        "{tag:?}",
      );

      assert_eq!(value.name, tag.name());
    }
  }

  #[test]
  fn inputs() {
    let reference = Reference::new();

    for command in reference.commands.subcommands.values() {
      if let Some(input) = &command.input {
        assert!(reference.json.contains_key(input), "{}", command.name);
      }
    }

    assert_eq!(
      reference.commands.subcommands["archive"].input.as_deref(),
      Some("Manifest"),
    );
  }

  #[test]
  fn languages() {
    let languages = Reference::new().languages;

    assert_eq!(languages.len(), language::CODES.len());

    assert_eq!(languages[&"en".parse::<Language>().unwrap()], "English");
  }

  #[test]
  fn lints() {
    let reference = Reference::new();

    assert_eq!(
      reference.lints[&Lint::Junk],
      "Package contains no junk files."
    );

    assert_eq!(
      reference.lint_groups[&crate::LintGroup::Hygiene],
      LintGroup {
        description: "Files that shouldn't be in a package.".into(),
        lints: BTreeSet::from([Lint::Junk]),
      },
    );
  }

  #[test]
  fn media_formats() {
    assert_eq!(
      Reference::new().media_formats[&MediaType::Audio],
      [
        MediaFormat {
          description: "[Free Lossless Audio Codec](https://en.wikipedia.org/wiki/FLAC).".into(),
          extension: "flac".into(),
          name: "FLAC".into(),
        },
        MediaFormat {
          description: "[MPEG-1 Audio Layer III](https://en.wikipedia.org/wiki/MP3).".into(),
          extension: "mp3".into(),
          name: "MP3".into(),
        },
        MediaFormat {
          description: "[MPEG-4 Part 14](https://en.wikipedia.org/wiki/MP4_file_format).".into(),
          extension: "m4a".into(),
          name: "MP4".into(),
        },
      ],
    );
  }

  #[test]
  fn outputs() {
    let reference = Reference::new();

    for command in reference.commands.subcommands.values() {
      if let Some(output) = &command.output {
        assert!(reference.json.contains_key(output), "{}", command.name);
      }
    }

    assert_eq!(
      reference.commands.subcommands["size"].output.as_deref(),
      Some("Totals"),
    );
  }

  #[test]
  fn parameters() {
    for (path, route) in Reference::new().routes {
      for endpoint in route.methods.values() {
        assert_eq!(endpoint.parameters.len(), route.parameters.len(), "{path}");
      }

      let mut parameters = route.methods.values().map(|endpoint| &endpoint.parameters);

      let first = parameters.next().unwrap();

      assert!(parameters.all(|parameters| parameters == first), "{path}");
    }

    let mut parameters = BTreeMap::<&str, Vec<PathParameter>>::new();

    for route in subcommand::serve::route::Route::all(&mut deco::Registry::new()) {
      if let Some(existing) = parameters.insert(route.path, route.parameters.clone()) {
        assert_eq!(existing, route.parameters, "{}", route.path);
      }
    }
  }

  #[test]
  fn unstable_features() {
    assert_eq!(
      Reference::new().unstable_features[&UnstableFeature::WebPackages],
      "Packages with `media: web` metadata.",
    );
  }
}
