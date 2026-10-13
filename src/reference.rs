use {super::*, clap::CommandFactory};

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
  pub about: String,
  pub help: String,
  pub name: String,
  pub subcommands: BTreeMap<String, Command>,
}

impl Command {
  fn new(command: &mut clap::Command) -> Self {
    Self {
      about: command.get_about().unwrap().to_string(),
      help: command.render_long_help().to_string(),
      name: command.get_name().into(),
      subcommands: command
        .get_subcommands_mut()
        .map(|command| (command.get_name().to_string(), Command::new(command)))
        .collect(),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Reference {
  pub commands: Command,
  pub languages: BTreeSet<Language>,
  pub lint_groups: BTreeMap<LintGroup, BTreeSet<Lint>>,
  pub lints: BTreeSet<Lint>,
  pub routes: BTreeMap<String, Route>,
  pub schemas: BTreeMap<String, Schema>,
  pub tags: BTreeMap<Tag, Hex>,
  pub unstable_features: BTreeSet<UnstableFeature>,
}

impl Reference {
  pub fn new() -> Self {
    fn tag<T: crate::Hex>(schemas: &mut BTreeMap<String, Schema>, tags: &mut BTreeMap<Tag, Hex>) {
      tags.insert(
        T::TAG,
        Hex {
          example: T::EXAMPLE.into(),
          schema: T::schema(schemas),
        },
      );
    }

    let mut command = Arguments::command();

    command.build();

    let mut schemas = BTreeMap::new();

    Archive::schema(&mut schemas);
    Directory::schema(&mut schemas);
    Envelope::<Claims>::schema(&mut schemas);
    Envelope::<Statement>::schema(&mut schemas);
    Metadata::schema(&mut schemas);
    RevisionObject::schema(&mut schemas);

    let mut tags = BTreeMap::new();

    tag::<Attestation>(&mut schemas, &mut tags);
    tag::<DisplayPrivateKey>(&mut schemas, &mut tags);
    tag::<Fingerprint>(&mut schemas, &mut tags);
    tag::<Hash>(&mut schemas, &mut tags);
    tag::<PublicKey>(&mut schemas, &mut tags);
    tag::<Revision>(&mut schemas, &mut tags);
    tag::<Token>(&mut schemas, &mut tags);

    let mut routes = BTreeMap::<String, Route>::new();

    for route in subcommand::serve::route::Route::all(&mut schemas) {
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
      commands: Command::new(&mut command),
      languages: language::ALL.clone(),
      lint_groups: LintGroup::iter()
        .map(|group| (group, group.lints()))
        .collect(),
      lints: Lint::iter().collect(),
      routes,
      schemas,
      tags,
      unstable_features: UnstableFeature::iter().collect(),
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
  pub schema: String,
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
  Deco { limit: u64, schema: Schema },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Response {
  Deco { schema: Schema },
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
  fn languages() {
    let languages = Reference::new().languages;

    assert_eq!(languages.len(), language::CODES.len());

    assert!(languages.contains(&"en".parse::<Language>().unwrap()));
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
  fn tags() {
    let tags = Reference::new().tags;

    assert_eq!(
      tags.keys().copied().collect::<Vec<Tag>>(),
      Tag::iter().collect::<Vec<Tag>>(),
    );

    for (tag, hex) in tags {
      assert!(
        hex.example.starts_with(&format!("{}1", tag.prefix())),
        "{tag:?}",
      );
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
