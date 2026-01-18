use super::*;

#[derive(Parser)]
#[group(required = true)]
pub(crate) struct Contains {
  #[arg(group = "target", help = "Search manifest for <FILE>.", long)]
  file: Option<Utf8PathBuf>,
  #[arg(group = "target", help = "Search manifest for <HASH>.", long)]
  hash: Option<Hash>,
  #[arg(help = MANIFEST_PATH_HELP)]
  manifest: Option<Utf8PathBuf>,
}

impl Contains {
  pub(crate) fn run(self, options: Options) -> Result {
    let manifest = Manifest::load(self.manifest.as_deref())?;

    let (hash, size) = if let Some(hash) = self.hash {
      (hash, None)
    } else {
      let path = self.file.unwrap();

      let file = options
        .hash_file(&path)
        .context(error::FilesystemIo { path })?;

      (file.hash, Some(file.size))
    };

    let file = manifest
      .files()
      .values()
      .find(|file| file.hash == hash)
      .copied()
      .context(error::FileNotFound { hash })?;

    if let Some(size) = size {
      ensure! {
        file.size == size,
        error::SizeMismatch {
          disk: size,
          hash,
          manifest: file.size,
        },
      }
    }

    Ok(())
  }
}
