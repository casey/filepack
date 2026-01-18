use super::*;

#[derive(Parser)]
pub(crate) struct Contains {
  #[arg(long, help = "Search manifest for file with <HASH>.")]
  hash: Hash,
  #[arg(help = MANIFEST_PATH_HELP)]
  manifest: Option<Utf8PathBuf>,
}

impl Contains {
  pub(crate) fn run(self) -> Result {
    let manifest = Manifest::load(self.manifest.as_deref())?;

    ensure! {
      manifest.files().values().any(|file| file.hash == self.hash),
      error::FileNotFound { hash: self.hash },
    }

    Ok(())
  }
}
