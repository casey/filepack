use super::*;

pub struct Loader {
  archive: Archive,
  options: DecodeOptions,
  path: Utf8PathBuf,
}

impl Loader {
  pub(crate) fn archive(&self) -> &Archive {
    &self.archive
  }

  pub fn fingerprint(&self) -> Result<Fingerprint> {
    self
      .archive
      .fingerprint_with_options(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }

  pub fn load(path: Option<&Utf8Path>) -> Result<Self> {
    Self::load_with_options(DecodeOptions::new(), path)
  }

  pub fn load_with_options(options: DecodeOptions, path: Option<&Utf8Path>) -> Result<Self> {
    let path = if let Some(path) = path {
      if path.is_dir() {
        path.join(Manifest::FILENAME)
      } else {
        path.into()
      }
    } else {
      Manifest::FILENAME.into()
    };

    let deco = filesystem::read_opt(&path)?
      .ok_or_else(|| error::ManifestNotFound { path: &path }.build())?;

    let archive = Archive::decode_from_slice_with_options(options, &deco)
      .context(error::DecodeManifest { path: &path })?;

    Ok(Self {
      archive,
      options,
      path,
    })
  }

  pub(crate) fn package(&self) -> Result<Entry> {
    self
      .archive
      .package(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }

  pub(crate) fn path(&self) -> &Utf8Path {
    &self.path
  }

  pub(crate) fn unpack(&self) -> Result<Manifest> {
    self
      .archive
      .unpack_with_options(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }

  pub(crate) fn unpack_with_totals(&self) -> Result<(Manifest, Totals)> {
    self
      .archive
      .unpack_with_totals(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }
}
