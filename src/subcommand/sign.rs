use super::*;

#[derive(Parser)]
pub(crate) struct Sign {
  #[arg(help = "Sign and update <MANIFEST>")]
  manifest: Utf8PathBuf,
}

impl Sign {
  pub(crate) fn run(self, options: Options) -> Result {
    let json = fs::read_to_string(&self.manifest).context(error::Io {
      path: &self.manifest,
    })?;

    let mut manifest =
      serde_json::from_str::<Manifest>(&json).context(error::DeserializeManifest {
        path: &self.manifest,
      })?;

    let root_hash = manifest.root_hash();

    for (public_key, signature) in &manifest.signatures {
      public_key.verify(root_hash.as_bytes(), signature)?;
    }

    let private_key_path = options.key_dir()?.join(MASTER_PRIVATE_KEY);

    let (public_key, signature) =
      PrivateKey::load_and_sign(&private_key_path, root_hash.as_bytes())?;

    manifest.signatures.insert(public_key, signature);

    fs::write(&self.manifest, manifest.to_json()).context(error::Io {
      path: &self.manifest,
    })?;

    Ok(())
  }
}
