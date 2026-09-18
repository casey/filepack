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
    let (path, archive) = Archive::load_with_opt_path(self.path.as_deref())?;

    let mut manifest = archive
      .unpack()
      .context(error::UnarchiveManifest { path: &path })?;

    let fingerprint = archive.fingerprint().unwrap();

    let keychain = Keychain::load(&options)?;

    manifest.sign(
      fingerprint,
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
