use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Package {
  pub(crate) fingerprint: Hash,
  pub(crate) manifest: Manifest,
  pub(crate) metadata: Option<Metadata>,
}

impl Package {
  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    let (_path, manifest) = Manifest::load(Some(path))?;

    let metadata_path = match path.parent() {
      Some(parent) => parent.join("metadata.json"),
      None => "metadata.json".into(),
    };

    let metadata = filesystem::read_to_string_opt(&metadata_path)?;

    let metadata = match metadata {
      Some(metadata) => Some(serde_json::from_str::<Metadata>(&metadata).context(
        error::DeserializeManifest {
          path: metadata_path,
        },
      )?),
      None => None,
    };

    Ok(Self {
      fingerprint: manifest.fingerprint(),
      manifest,
      metadata,
    })
  }
}
