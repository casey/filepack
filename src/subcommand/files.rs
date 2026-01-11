use super::*;

#[derive(Parser)]
pub(crate) struct Files {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Files {
  pub(crate) fn run(self) -> Result {
    let (_path, manifest) = Manifest::load(self.path.as_deref())?;

    println!(
      "{}",
      serde_json::to_string_pretty(&manifest.files()).unwrap(),
    );

    Ok(())
  }
}
