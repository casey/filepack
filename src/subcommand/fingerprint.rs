use super::*;

#[derive(Parser)]
pub(crate) struct Fingerprint {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Fingerprint {
  pub(crate) fn run(self) -> Result {
    let manifest = Manifest::load(self.path.as_deref())?;

    println!("{}", manifest.fingerprint());

    Ok(())
  }
}
