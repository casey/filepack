use {super::*, clap::CommandFactory};

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
  pub about: String,
  pub help: String,
  pub name: String,
  pub output: Option<String>,
  pub subcommands: BTreeMap<String, Command>,
}

impl Command {
  fn new(command: &mut clap::Command, registry: &mut json::Registry) -> Self {
    Self {
      about: command.get_about().unwrap().to_string(),
      help: command.render_long_help().to_string(),
      name: command.get_name().into(),
      output: Self::output(command, registry),
      subcommands: command
        .get_subcommands_mut()
        .map(|command| {
          (
            command.get_name().to_string(),
            Command::new(command, registry),
          )
        })
        .collect(),
    }
  }

  fn output(command: &clap::Command, registry: &mut json::Registry) -> Option<String> {
    match command.get_name() {
      "files" => Some(BTreeMap::<RelativePath, File>::json_schema(registry)),
      "languages" => Some(BTreeMap::<Language, String>::json_schema(registry)),
      "lints" => Some(BTreeMap::<LintGroup, BTreeSet<Lint>>::json_schema(registry)),
      "manifest" => Some(Manifest::json_schema(registry)),
      "signatures" => Some(Vec::<SignatureEntry>::json_schema(registry)),
      "size" => Some(Totals::json_schema(registry)),
      _ => None,
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
  pub lint_groups: BTreeMap<LintGroup, BTreeSet<Lint>>,
  pub lints: BTreeSet<Lint>,
  pub media_formats: BTreeMap<MediaType, Vec<MediaFormat>>,
  pub routes: BTreeMap<String, Route>,
  pub unstable_features: BTreeSet<UnstableFeature>,
  pub yaml: json::Registry,
}

impl Reference {
  pub fn new() -> Self {
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

    let commands = Command::new(&mut command, &mut json);

    let mut yaml = json::Registry::new();

    yaml::Metadata::json_schema(&mut yaml);

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
      lint_groups: LintGroup::iter()
        .map(|group| (group, group.lints()))
        .collect(),
      lints: Lint::iter().collect(),
      media_formats: [
        (MediaType::Audio, media_formats::<AudioFormat>()),
        (MediaType::Image, media_formats::<ImageFormat>()),
        (MediaType::Video, media_formats::<VideoFormat>()),
      ]
      .into(),
      routes,
      unstable_features: UnstableFeature::iter().collect(),
      yaml,
    }
  }
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Endpoint {
  pub authenticated: bool,
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
  Deco { limit: u64, schema: deco::Schema },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Response {
  Deco { schema: deco::Schema },
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
  fn languages() {
    let languages = Reference::new().languages;

    assert_eq!(languages.len(), language::CODES.len());

    assert_eq!(languages[&"en".parse::<Language>().unwrap()], "English");
  }

  #[test]
  fn lints() {
    let reference = Reference::new();

    assert!(reference.lints.contains(&Lint::Junk));

    assert_eq!(
      reference.lint_groups[&LintGroup::Hygiene],
      BTreeSet::from([Lint::Junk]),
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
