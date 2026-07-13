use super::*;

#[derive(Parser)]
pub(crate) struct Size {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Size {
  pub(crate) fn run(self) -> Result {
    let (path, archive) = Archive::load_with_opt_path(self.path.as_deref())?;

    archive
      .unpack()
      .context(error::UnarchiveManifest { path: &path })?;

    let package = archive
      .package()
      .context(error::UnarchiveManifest { path: &path })?;

    let Entry::Directory { totals, .. } = package else {
      unreachable!();
    };

    println!("{}", totals.file_size);

    Ok(())
  }
}
