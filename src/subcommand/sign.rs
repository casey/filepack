use super::*;

#[derive(Parser)]
pub(crate) struct Sign {
  #[arg(help = "Sign contents of <FILE>, defaulting to standard input")]
  file: Utf8PathBuf,
}

impl Sign {
  pub(crate) fn run(self, options: Options) -> Result {
    let message = fs::read(&self.file).context(error::Io { path: &self.file })?;

    let private_key_path = options.key_dir()?.join(MASTER_PRIVATE_KEY);

    let (public_key, signature) = PrivateKey::load_and_sign(&private_key_path, &message)?;

    let path = self
      .file
      .parent()
      .unwrap()
      .join(format!("{public_key}.signature"));

    fs::write(&path, signature.to_string()).context(error::Io { path })?;

    Ok(())
  }
}
