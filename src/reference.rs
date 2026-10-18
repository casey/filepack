use {super::*, clap::CommandFactory};

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
  pub about: String,
  pub help: String,
  pub input: Option<String>,
  pub name: String,
  pub output: Option<String>,
  pub subcommands: BTreeMap<String, Command>,
}

impl Command {
  fn new(
    command: &mut clap::Command,
    name: Option<subcommand::Name>,
    registry: &mut json::Registry,
  ) -> Self {
    Self {
      about: command.get_about().unwrap().to_string(),
      help: command.render_long_help().to_string(),
      input: name.and_then(|name| name.input(registry)),
      name: command.get_name().into(),
      output: name.and_then(|name| name.output(registry)),
      subcommands: command
        .get_subcommands_mut()
        .filter(|command| command.get_name() != "help")
        .map(|command| {
          let name = command.get_name().parse::<subcommand::Name>().unwrap();
          (
            command.get_name().to_string(),
            Command::new(command, Some(name), registry),
          )
        })
        .collect(),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Reference {
  pub commands: Command,
  pub deco: deco::Registry,
  pub hex: BTreeMap<Tag, Hex>,
  pub json: json::Registry,
  pub languages: BTreeMap<Language, String>,
  pub lint_groups: BTreeMap<crate::LintGroup, LintGroup>,
  pub lints: BTreeMap<Lint, String>,
  pub media_formats: BTreeMap<MediaType, Vec<MediaFormat>>,
  pub metadata: yaml::Example,
  pub routes: BTreeMap<String, Route>,
  pub unstable_features: BTreeSet<UnstableFeature>,
  pub yaml: json::Registry,
}

impl Reference {
  pub fn new() -> Self {
    fn descriptions(registry: &json::Registry, name: &str) -> BTreeMap<String, String> {
      let json::Schema::StringEnum { values, .. } = &registry[name] else {
        unreachable!()
      };

      values
        .iter()
        .map(|value| (value.value.clone(), value.description.clone()))
        .collect()
    }

    fn media_formats<F: Format>() -> Vec<MediaFormat> {
      let mut formats = BTreeMap::new();

      for extension in F::EXTENSIONS {
        let name = F::from_extension(extension).unwrap().to_string();

        assert!(
          formats
            .insert(name.clone(), (*extension).to_string())
            .is_none(),
          "format `{name}` has multiple extensions",
        );
      }

      formats
        .into_iter()
        .map(|(name, extension)| MediaFormat { extension, name })
        .collect()
    }

    fn tag<T: crate::Hex>(registry: &mut deco::Registry, hex: &mut BTreeMap<Tag, Hex>) {
      hex.insert(
        T::TAG,
        Hex {
          example: T::EXAMPLE.into(),
          name: T::TAG.name().into(),
          schema: T::deco_schema(registry),
        },
      );
    }

    let mut command = Arguments::command();

    command.build();

    let mut deco = BTreeMap::new();

    Archive::deco_schema(&mut deco);
    Directory::deco_schema(&mut deco);
    Envelope::<Claims>::deco_schema(&mut deco);
    Envelope::<Statement>::deco_schema(&mut deco);
    Metadata::deco_schema(&mut deco);
    RevisionObject::deco_schema(&mut deco);

    let mut hex = BTreeMap::new();

    tag::<Attestation>(&mut deco, &mut hex);
    tag::<DisplayPrivateKey>(&mut deco, &mut hex);
    tag::<Fingerprint>(&mut deco, &mut hex);
    tag::<Hash>(&mut deco, &mut hex);
    tag::<PublicKey>(&mut deco, &mut hex);
    tag::<Revision>(&mut deco, &mut hex);
    tag::<Token>(&mut deco, &mut hex);

    let mut json = json::Registry::new();

    let commands = Command::new(&mut command, None, &mut json);

    let lint_groups = descriptions(&json, "LintGroup");

    let lints = descriptions(&json, "Lint");

    let mut yaml = json::Registry::new();

    yaml::Metadata::json_schema(&mut yaml);

    let metadata = yaml::Example::new(&yaml, "Metadata");

    let mut routes = BTreeMap::<String, Route>::new();

    for route in subcommand::serve::route::Route::all(&mut deco) {
      routes
        .entry(route.path.into())
        .or_insert_with(|| {
          let mut parameters = Vec::new();
          let mut wildcard = false;

          for segment in route.path.split('/') {
            let Some(name) = segment.strip_prefix('{') else {
              continue;
            };

            let name = if let Some(name) = name.strip_prefix('*') {
              wildcard = true;
              name
            } else {
              name
            };

            parameters.push(name.strip_suffix('}').unwrap().into());
          }

          Route {
            api: route.path.starts_with("/api/"),
            methods: BTreeMap::new(),
            parameters,
            wildcard,
          }
        })
        .methods
        .insert(route.method.to_string(), route.endpoint);
    }

    Self {
      commands,
      deco,
      hex,
      json,
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
      unstable_features: UnstableFeature::iter().collect(),
      yaml,
    }
  }
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Endpoint {
  pub authenticated: bool,
  pub description: String,
  pub parameters: Vec<Parameter>,
  pub query: Vec<QueryParameter>,
  pub range: bool,
  pub request: Option<Request>,
  pub response: Option<Response>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Hex {
  pub example: String,
  pub name: String,
  pub schema: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct LintGroup {
  pub description: String,
  pub lints: BTreeSet<Lint>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct MediaFormat {
  pub extension: String,
  pub name: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Parameter {
  Fingerprint,
  Hash,
  Integer,
  Ordinal,
  PackageIdentifier,
  RelativePath,
  Revision,
  String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct QueryParameter {
  pub default: String,
  pub name: String,
  pub values: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Request {
  Bytes,
  Deco {
    fields: Vec<deco::Field>,
    limit: u64,
  },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Response {
  Deco { fields: Vec<deco::Field> },
  File,
  Html,
  Redirect,
  Static,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Route {
  pub api: bool,
  pub methods: BTreeMap<String, Endpoint>,
  pub parameters: Vec<String>,
  pub wildcard: bool,
}

#[cfg(test)]
mod tests {
  use super::*;

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
          extension: "flac".into(),
          name: "FLAC".into(),
        },
        MediaFormat {
          extension: "mp3".into(),
          name: "MP3".into(),
        },
        MediaFormat {
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
      assert_eq!(route.wildcard, path.contains("{*"), "{path}");

      for endpoint in route.methods.values() {
        assert_eq!(endpoint.parameters.len(), route.parameters.len(), "{path}");
      }

      let mut parameters = route.methods.values().map(|endpoint| &endpoint.parameters);

      let first = parameters.next().unwrap();

      assert!(parameters.all(|parameters| parameters == first), "{path}");
    }
  }

  #[test]
  fn unstable_features() {
    assert!(
      Reference::new()
        .unstable_features
        .contains(&UnstableFeature::WebPackages)
    );
  }
}
