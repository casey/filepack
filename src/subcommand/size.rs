use super::*;

#[derive(Parser)]
pub(crate) struct Size {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Size {
  pub(crate) fn run(self) -> Result {
    let (path, archive) = Archive::load_with_opt_path(self.path.as_deref())?;

    let (_manifest, totals) = archive
      .unpack_with_totals()
      .context(error::UnarchiveManifest { path: &path })?;

    println!("{}", totals.file_size);

    Ok(())
  }
}
