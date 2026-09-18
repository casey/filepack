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
    let loader = Loader::load_with_options(DecodeOptions::strict(), self.path.as_deref())?;

    let mut manifest = loader.unpack()?;

    let fingerprint = loader.fingerprint()?;

    let keychain = Keychain::load(&options)?;

    manifest.sign(
      fingerprint,
      SignOptions {
        timestamp: self.timestamp,
      },
      &keychain,
      &self.key,
    )?;

    manifest.save(loader.path())?;

    Ok(())
  }
}
