use {super::*, clap::CommandFactory};

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
  pub help: String,
  pub name: String,
  pub subcommands: BTreeMap<String, Command>,
}

impl Command {
  fn new(command: &mut clap::Command) -> Self {
    Self {
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
  pub routes: BTreeMap<String, Route>,
  pub schemas: BTreeMap<String, Schema>,
  pub tags: BTreeMap<String, Tag>,
}

impl Reference {
  pub fn new() -> Self {
    fn tag<T: Hex>(schemas: &mut BTreeMap<String, Schema>, tags: &mut BTreeMap<String, Tag>) {
      tags.insert(
        T::TAG.prefix().into(),
        Tag {
          example: T::EXAMPLE.into(),
          schema: T::schema(schemas),
        },
      );
    }

    let mut command = Arguments::command();

    command.build();

    let mut routes = BTreeMap::<String, Route>::new();

    for route in subcommand::serve::route::Route::all() {
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

    let mut schemas = BTreeMap::new();

    Application::schema(&mut schemas);
    Archive::schema(&mut schemas);
    Audio::schema(&mut schemas);
    AudioCodec::schema(&mut schemas);
    AudioFormat::schema(&mut schemas);
    CheckedUrl::schema(&mut schemas);
    ChromaSubsampling::schema(&mut schemas);
    Claims::schema(&mut schemas);
    ColorType::schema(&mut schemas);
    ComponentBuf::schema(&mut schemas);
    Context::schema(&mut schemas);
    Dimensions::schema(&mut schemas);
    Directory::schema(&mut schemas);
    Ed25519Signature::schema(&mut schemas);
    Entry::schema(&mut schemas);
    EntryInfo::schema(&mut schemas);
    Envelope::<Claims>::schema(&mut schemas);
    Envelope::<Statement>::schema(&mut schemas);
    Image::schema(&mut schemas);
    ImageFormat::schema(&mut schemas);
    Language::schema(&mut schemas);
    Media::schema(&mut schemas);
    Metadata::schema(&mut schemas);
    Orientation::schema(&mut schemas);
    PackageMetadata::schema(&mut schemas);
    RelativePath::schema(&mut schemas);
    RevisionObject::schema(&mut schemas);
    Rotation::schema(&mut schemas);
    Statement::schema(&mut schemas);
    String::schema(&mut schemas);
    Text::schema(&mut schemas);
    Time::schema(&mut schemas);
    Totals::schema(&mut schemas);
    Track::schema(&mut schemas);
    TrackCodec::schema(&mut schemas);
    TrackInfo::schema(&mut schemas);
    Vec::<u8>::schema(&mut schemas);
    Video::schema(&mut schemas);
    VideoFormat::schema(&mut schemas);
    bool::schema(&mut schemas);
    i64::schema(&mut schemas);
    u64::schema(&mut schemas);

    let mut tags = BTreeMap::new();

    tag::<Attestation>(&mut schemas, &mut tags);
    tag::<DisplayPrivateKey>(&mut schemas, &mut tags);
    tag::<Fingerprint>(&mut schemas, &mut tags);
    tag::<Hash>(&mut schemas, &mut tags);
    tag::<PublicKey>(&mut schemas, &mut tags);
    tag::<Revision>(&mut schemas, &mut tags);
    tag::<Token>(&mut schemas, &mut tags);

    Self {
      commands: Command::new(&mut command),
      routes,
      schemas,
      tags,
    }
  }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Endpoint {
  pub authenticated: bool,
  pub parameters: Vec<Parameter>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Route {
  pub api: bool,
  pub methods: BTreeMap<String, Endpoint>,
  pub parameters: Vec<String>,
  pub wildcard: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
  pub example: String,
  pub schema: String,
}

#[cfg(test)]
mod tests {
  use super::*;

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
  }

  #[test]
  fn tags() {
    let tags = Reference::new().tags;

    assert_eq!(
      tags.keys().cloned().collect::<Vec<String>>(),
      tag::Tag::iter()
        .map(|tag| tag.prefix().into())
        .collect::<Vec<String>>(),
    );

    for (prefix, tag) in tags {
      assert!(tag.example.starts_with(&format!("{prefix}1")), "{prefix}");
    }
  }
}
