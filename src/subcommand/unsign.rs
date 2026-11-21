use super::*;

#[derive(Parser)]
#[command(group = ArgGroup::new("target").required(true))]
pub(crate) struct Unsign {
  #[arg(group = "target", help = "Remove all signatures", long)]
  all: bool,
  #[arg(
    group = "target",
    help = "Remove signature by <KEY>",
    long = "key",
    value_name = "KEY"
  )]
  keys: Vec<KeyIdentifier>,
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Unsign {
  pub(crate) fn run(self, options: Options) -> Result {
    let loader = Loader::load_with_options(DecodeOptions::strict(), self.path.as_deref())?;

    let mut manifest = loader.unpack()?;

    if self.all {
      manifest.signatures.clear();
    } else {
      for (key, identifier) in Keychain::resolve(&options, &self.keys)? {
        let count = manifest.signatures.len();

        manifest.signatures.retain(|signature| {
          signature
            .known()
            .is_none_or(|signature| signature.public_key() != key)
        });

        ensure! {
          manifest.signatures.len() < count,
          error::SignatureMissing { identifier: identifier.clone() },
        }
      }
    }

    manifest.save(loader.path())?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_conflicts_with_key() {
    assert_argument_conflict::<Unsign>(&["--all", "--key", "foo"], "--all", "--key <KEY>");
  }

  #[test]
  fn target_is_required() {
    assert_missing_argument::<Unsign>(&[], &["<--all|--key <KEY>>"]);
  }
}
