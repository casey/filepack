use super::*;

#[derive(Parser)]
pub(crate) struct Fingerprint {
  #[arg(
    help = "Load manifest from <PATH>. May be path to manifest, to directory containing manifest \
    named `filepack.json`, or omitted, in which case manifest named `filepack.json` in the current \
    directory is loaded."
  )]
  path: Option<Utf8PathBuf>,
}

impl Fingerprint {
  pub(crate) fn run(self) -> Result {
    let (_path, manifest) = Manifest::load(self.path.as_deref())?;

    println!("{}", manifest.fingerprint());

    Ok(())
  }
}
