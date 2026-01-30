use super::*;

#[derive(Parser)]
pub(crate) struct Sign {
  #[arg(default_value_t = KeyName::DEFAULT, help = "Sign with <KEY>", long)]
  key: KeyName,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
  #[arg(help = TIME_HELP, long)]
  time: bool,
}

impl Sign {
  pub(crate) fn run(self, options: Options) -> Result {
    let (path, mut manifest) = Manifest::load_with_path(self.path.as_deref())?;

    let keychain = Keychain::load(&options)?;

    manifest.sign(SignOptions { time: self.time }, &keychain, &self.key)?;

    manifest.save(&path)?;

    Ok(())
  }
}
