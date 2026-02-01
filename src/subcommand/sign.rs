use super::*;

#[derive(Parser)]
pub(crate) struct Sign {
  #[arg(default_value_t = KeyName::DEFAULT, help = "Sign with <KEY>", long)]
  key: KeyName,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
  #[arg(help = TIMESTAMP_HELP, long)]
  timestamp: bool,
}

impl Sign {
  pub(crate) fn run(self, options: Options) -> Result {
    let (path, mut manifest) = Manifest::load_with_path(self.path.as_deref())?;

    let keychain = Keychain::load(&options)?;

    manifest.sign(
      SignOptions {
        timestamp: self.timestamp,
      },
      &keychain,
      &self.key,
    )?;

    manifest.save(&path)?;

    Ok(())
  }
}
