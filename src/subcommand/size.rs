use super::*;

#[derive(Parser)]
pub(crate) struct Size {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Size {
  pub(crate) fn run(self) -> Result {
    let (path, manifest) = Manifest::load_with_opt_path(self.path.as_deref())?;
    println!(
      "{}",
      manifest
        .package
        .totals()
        .context(error::ManifestTotals { path })?
        .file_size
    );
    Ok(())
  }
}
