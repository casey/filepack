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
  pub routes: BTreeMap<String, BTreeMap<String, Endpoint>>,
  pub schemas: BTreeMap<String, Schema>,
  pub tags: BTreeMap<String, String>,
}

impl Reference {
  pub fn new() -> Self {
    let mut command = Arguments::command();

    command.build();

    let mut routes = BTreeMap::<String, BTreeMap<String, Endpoint>>::new();

    for route in Route::all() {
      routes
        .entry(route.path.into())
        .or_default()
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

    for (tag, schema) in [
      (Attestation::TAG, Attestation::schema(&mut schemas)),
      (
        DisplayPrivateKey::TAG,
        DisplayPrivateKey::schema(&mut schemas),
      ),
      (Fingerprint::TAG, Fingerprint::schema(&mut schemas)),
      (Hash::TAG, Hash::schema(&mut schemas)),
      (PublicKey::TAG, PublicKey::schema(&mut schemas)),
      (Revision::TAG, Revision::schema(&mut schemas)),
      (Token::TAG, Token::schema(&mut schemas)),
    ] {
      tags.insert(tag.prefix().into(), schema);
    }

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
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tags() {
    assert_eq!(
      Reference::new().tags.into_keys().collect::<Vec<String>>(),
      Tag::iter()
        .map(|tag| tag.prefix().into())
        .collect::<Vec<String>>(),
    );
  }
}
