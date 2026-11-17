use super::*;

#[derive(Parser)]
pub(crate) struct Sign {
  #[arg(default_value_t = KeyName::DEFAULT, help = "Sign with <KEY>", long)]
  key: KeyName,
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Sign {
  pub(crate) fn run(self, options: Options) -> Result {
    let loader = Loader::load_with_options(DecodeOptions::strict(), self.path.as_deref())?;

    let mut manifest = loader.unpack()?;

    let revision_hash = loader.revision_hash()?;

    let keychain = Keychain::load(&options)?;

    manifest.sign(&keychain, &self.key, revision_hash)?;

    let archive = manifest.pack(loader.path())?;

    let actual =
      archive
        .revision_hash(DecodeOptions::strict())
        .context(error::UnarchiveManifest {
          path: loader.path(),
        })?;

    ensure! {
      actual == revision_hash,
      error::Internal {
        message: format!("signing changed revision hash from {revision_hash} to {actual}"),
      },
    }

    filesystem::write_atomic(loader.path(), archive.encode_to_vec())?;

    Ok(())
  }
}
