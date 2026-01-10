use super::*;

// todo:
// - figure out if I'm using the right paths in errors
// - make sure signatures are saved with trailing newline

#[derive(Parser)]
pub(crate) struct Sign {
  #[arg(help = "Allow overwriting signature", long)]
  force: bool,
  #[arg(
    help = "Sign <ROOT>. May be a path to a manifest or a directory containing a manifest named \
    `filepack.json`. If omitted, the manifest `filepack.json` in the current directory is signed."
  )]
  root: Option<Utf8PathBuf>,
}

impl Sign {
  pub(crate) fn run(self, options: Options) -> Result {
    let (path, manifest) = Manifest::load(self.root.as_deref())?;

    let signatures = path.parent().unwrap().join(SIGNATURES_DIRECTORY);

    if !self.force {
      let public_key_path = options.key_dir()?.join(MASTER_PUBLIC_KEY);
      let public_key = PublicKey::load(&public_key_path)?;
      ensure! {
        !filesystem::exists(&public_key.signature_path(&signatures))?,
        error::SignatureAlreadyExists { public_key },
      }
    }

    let private_key_path = options.key_dir()?.join(MASTER_PRIVATE_KEY);

    let fingerprint = manifest.fingerprint();

    let (public_key, signature) = PrivateKey::load_and_sign(&private_key_path, fingerprint)?;

    filesystem::create_dir_all(&signatures)?;

    filesystem::write(
      &public_key.signature_path(&signatures),
      signature.to_string(),
    )?;

    Ok(())
  }
}
