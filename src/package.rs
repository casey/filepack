use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Package {
  pub(crate) hash: Hash,
  pub(crate) manifest: Manifest,
  pub(crate) metadata: Option<Metadata>,
}

impl Package {
  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    let (_path, manifest) = Manifest::load(Some(path))?;

    let metadata_path = path.parent().unwrap().join("metadata.json");

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
      hash: manifest.fingerprint(),
      manifest,
      metadata,
    })
  }
}
