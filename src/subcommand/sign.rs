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

    for note in &manifest.notes {
      for (public_key, signature) in &note.signatures {
        public_key.verify(fingerprint, signature)?;
      }
    }

    let keychain = Keychain::load(&options)?;

    let message = Message { fingerprint };

    let (public_key, signature) = keychain.sign_message(&self.key, message)?;

    ensure! {
      self.force || manifest.notes.iter().all(|note| !note.has_signature(public_key)),
      error::SignatureAlreadyExists { public_key: public_key.clone() },
    }

    let note = Note {
      signatures: [(public_key.clone(), signature)].into(),
    };

    manifest.notes.push(note);

    manifest.save(&path)?;

    Ok(())
  }
}
