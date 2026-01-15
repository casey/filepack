use super::*;

#[derive(Default)]
pub(crate) struct Keys {
  pub(crate) public_keys: BTreeMap<KeyName, PublicKey>,
}

impl Keys {
  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    if !filesystem::exists(path)? {
      return Ok(Self::default());
    }

    let mode = filesystem::mode(path)?;

    ensure! {
      mode.is_secure(),
      error::KeyDirPermissions { mode, path },
    }

    let mut public_keys = BTreeMap::new();
    for entry in WalkDir::new(path).max_depth(1) {
      let entry = entry?;

      if entry.depth() == 0 {
        continue;
      }

      let path = decode_path(entry.path())?;

      if path.file_name().unwrap().starts_with('.') {
        continue;
      }

      if !entry.file_type().is_file() {
        return Err(error::KeyDirUnexpectedDirectory { path }.build());
      }

      let Some(extension) = path.extension() else {
        return Err(error::KeyDirUnexpectedFile { path }.build());
      };

      let key_type = extension
        .parse::<KeyType>()
        .ok()
        .context(error::KeyDirUnexpectedFile { path })?;

      let stem = path.file_stem().unwrap();

      let name = stem
        .parse::<KeyName>()
        .context(error::KeyDirKeyName { path })?;

      match key_type {
        KeyType::Private => {
          let mode = filesystem::mode(path)?;

          ensure! {
            mode.is_secure(),
            error::PrivateKeyPermissions { mode, path },
          }

          let path = path.with_file_name(name.public_key_filename());

          ensure! {
            filesystem::exists(&path)?,
            error::KeyDirPublicKeyNotFound { path },
          }
        }
        KeyType::Public => {
          let key = PublicKey::load(path)?;

          let path = path.with_file_name(name.private_key_filename());

          ensure! {
            filesystem::exists(&path)?,
            error::KeyDirPrivateKeyNotFound { path },
          }

          public_keys.insert(name, key);
        }
      }
    }

    Ok(Self { public_keys })
  }
}
