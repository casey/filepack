use super::*;

#[derive(Parser)]
pub(crate) struct Fingerprint {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Fingerprint {
  pub(crate) fn run(self) -> Result {
    let (path, archive) = Archive::load_with_opt_path(self.path.as_deref())?;

    archive
      .unpack()
      .context(error::UnarchiveManifest { path: &path })?;

    let fingerprint = archive
      .fingerprint()
      .context(error::UnarchiveManifest { path: &path })?;

    println!("{fingerprint}");

    Ok(())
  }
}
