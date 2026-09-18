use super::*;

#[derive(Parser)]
pub(crate) struct Fingerprint {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Fingerprint {
  pub(crate) fn run(self) -> Result {
    let loader = Loader::load(self.path.as_deref())?;

    loader.unpack()?;

    println!("{}", loader.fingerprint()?);

    Ok(())
  }
}
