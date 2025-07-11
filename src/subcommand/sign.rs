use super::*;

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
    let (path, _json, mut manifest) = Manifest::load(self.root.as_deref())?;

    let fingerprint = manifest.fingerprint();

    for (public_key, signature) in &manifest.signatures {
      public_key.verify(fingerprint.as_bytes(), signature)?;
    }

    if !self.force {
      let public_key_path = options.key_dir()?.join(MASTER_PUBLIC_KEY);
      let public_key = PublicKey::load(&public_key_path)?;
      ensure! {
        !manifest.signatures.contains_key(&public_key),
        error::SignatureAlreadyExists { public_key },
      }
    }

    let private_key_path = options.key_dir()?.join(MASTER_PRIVATE_KEY);

    let (public_key, signature) =
      PrivateKey::load_and_sign(&private_key_path, fingerprint.as_bytes())?;

    manifest.signatures.insert(public_key, signature);

    manifest.store(&path)?;

    Ok(())
  }
}
