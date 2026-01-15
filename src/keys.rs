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

    let mode = filesystem::mode(&path)?;

    ensure! {
      mode.is_secure(),
      error::KeyDirPermissions { mode, path },
    }

    let mut public_keys = BTreeMap::new();
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
        return Err(error::KeyDirDirectory { path }.build());
      }

      let Some(extension) = path.extension() else {
        return Err(error::KeyDirExtension { path }.build());
      };

      let Ok(key_type) = extension.parse::<KeyType>() else {
        return Err(
          error::KeyDirType {
            extension: extension.to_string(),
            path,
          }
          .build(),
        );
      };

      let stem = path.file_stem().unwrap();

      let name = stem.parse::<KeyName>().context(error::KeyNameInvalid {
        stem: stem.to_string(),
      })?;

      match key_type {
        KeyType::Private => {
          let mode = filesystem::mode(path)?;

          ensure! {
            mode.is_secure(),
            error::PrivateKeyPermissions { mode, path },
          }

          let public_key_path = path.with_file_name(name.public_key_filename());

          ensure! {
            filesystem::exists(&public_key_path)?,
            error::PublicKeyNotFound { path: public_key_path },
          }

          PublicKey::load(&public_key_path)?;
        }
        KeyType::Public => {
          let public_key = PublicKey::load(path)?;

          let private_key_path = path.with_file_name(name.private_key_filename());

          ensure! {
            filesystem::exists(&private_key_path)?,
            error::PrivateKeyNotFound { path: private_key_path },
          }

          public_keys.insert(name, public_key);
        }
      }
    }

    Ok(Self { public_keys })
  }
}
