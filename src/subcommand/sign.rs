use super::*;

#[derive(Parser)]
pub(crate) struct Sign {
  #[arg(help = "Allow overwriting signature", long)]
  force: bool,
  #[arg(default_value_t = KeyName::DEFAULT, help = "Sign with <KEY>", long)]
  key: KeyName,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Sign {
  pub(crate) fn run(self, options: Options) -> Result {
    let (path, mut manifest) = Manifest::load_with_path(self.path.as_deref())?;

    let fingerprint = manifest.fingerprint();

    for (public_key, signature) in &manifest.signatures {
      public_key.verify(fingerprint, signature)?;
    }

    let public_key_path = options.key_dir()?.join(self.key.public_key_filename());

    let public_key = PublicKey::load(&public_key_path)?;

    ensure! {
      self.force || !manifest.signatures.contains_key(&public_key),
      error::SignatureAlreadyExists { public_key },
    }

    let private_key_path = options.key_dir()?.join(self.key.private_key_filename());

    let signature =
      PrivateKey::load_and_sign(&self.key, &public_key, &private_key_path, fingerprint)?;

    manifest.signatures.insert(public_key, signature);

    manifest.save(&path)?;

    Ok(())
  }
}
