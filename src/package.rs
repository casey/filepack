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

    Ok(Self {
      hash: manifest.fingerprint(),
      manifest,
      metadata: None,
    })
  }
}
