use super::*;

#[derive(Default)]
pub(crate) struct Keys {
  pub(crate) names: BTreeMap<KeyName, PublicKey>,
}

impl Keys {
  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    if !filesystem::exists(path)? {
      return Ok(Self::default());
    }

    let mode = filesystem::mode(&path)?;

    ensure! {
      mode.is_secure(),
      error::KeyDirPermissions { mode, path },
    }

    let mut names = BTreeMap::new();

    for entry in WalkDir::new(&path).max_depth(1) {
      let entry = entry?;

      if entry.depth() == 0 {
        continue;
      }

      let path = decode_path(entry.path())?;

      if path.file_name().unwrap().starts_with('.') {
        continue;
      }

      if !entry.file_type().is_file() {
        panic!();
      }

      let Some(extension) = path.extension() else {
        panic!();
      };

      if extension == PRIVATE_KEY_EXTENSION {
        let mode = filesystem::mode(path)?;

        ensure! {
          mode.is_secure(),
          error::PrivateKeyPermissions { mode, path },
        }

        // todo:
        // - check that public key exists
        // - check that public key corresponds to private key

        continue;
      }

      if extension != PUBLIC_KEY_EXTENSION {
        panic!();
      }

      // todo:
      // - check that private key exists

      let name = path.file_stem().unwrap().parse::<KeyName>().unwrap();

      let public_key = PublicKey::load(path)?;

      names.insert(name, public_key);
    }

    Ok(Self { names })
  }
}
