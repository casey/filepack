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
        todo!("unexpected directory");
      }

      let Some(extension) = path.extension() else {
        todo!("unexpected file");
      };

      let key_type = extension
        .parse::<KeyType>()
        .expect("todo: unexpected file type");

      let name = path
        .file_stem()
        .unwrap()
        .parse::<KeyName>()
        .expect("todo: bad key name");

      match key_type {
        KeyType::Private => {
          let mode = filesystem::mode(path)?;

          ensure! {
            mode.is_secure(),
            error::PrivateKeyPermissions { mode, path },
          }

          let public_key = path.with_file_name(name.public_key_filename());

          if !filesystem::exists(&public_key)? {
            todo!();
          }

          PublicKey::load(&public_key)?;
        }
        KeyType::Public => {
          let public_key = PublicKey::load(path)?;

          let private_key = path.with_file_name(name.private_key_filename());

          if !filesystem::exists(&private_key)? {
            todo!()
          }

          public_keys.insert(name, public_key);
        }
      }
    }

    Ok(Self { public_keys })
  }
}
