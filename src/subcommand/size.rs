use super::*;

#[derive(Parser)]
pub(crate) struct Size {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Size {
  pub(crate) fn run(self) -> Result {
    let (_path, manifest) = Manifest::load(self.path.as_deref())?;
    println!("{}", manifest.total_size());
    Ok(())
  }
}
