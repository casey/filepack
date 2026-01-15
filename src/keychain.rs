use super::*;

#[derive(Default)]
pub(crate) struct Keychain {
  pub(crate) path: Utf8PathBuf,
  pub(crate) keys: BTreeMap<KeyName, PublicKey>,
}

impl Keychain {
  pub(crate) fn load(options: &Options) -> Result<Self> {
    let path = options.key_dir()?;

    if !filesystem::exists(&path)? {
      return Ok(Self::default());
    }

    let mode = filesystem::mode(&path)?;

    ensure! {
      mode.is_secure(),
      error::KeyDirPermissions { mode, path },
    }

    let mut keys = BTreeMap::new();
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

      let name = stem.parse::<KeyName>().context(error::KeyName { path })?;

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
            error::PublicKeyNotFound { path },
          }
        }
        KeyType::Public => {
          let key = PublicKey::load(path)?;

          let path = path.with_file_name(name.private_key_filename());

          ensure! {
            filesystem::exists(&path)?,
            error::PrivateKeyNotFound { path },
          }

          keys.insert(name, key);
        }
      }
    }

    Ok(Self {
      keys,
      path: path.into(),
    })
  }

  pub(crate) fn public_key(&self, name: &KeyName) -> Result<&PublicKey> {
    self.keys.get(name).context(error::PublicKeyNotFound {
      path: self.path.join(name.public_key_filename()),
    })
  }

  pub(crate) fn sign(&self, name: &KeyName, fingerprint: Hash) -> Result<(&PublicKey, Signature)> {
    let public_key = self.public_key(name)?;

    let private_key = PrivateKey::load(&self.path.join(name.private_key_filename()))?;

    ensure! {
      private_key.public_key() == *public_key,
      error::KeyMismatch {
        key: name.clone(),
      }
    }

    Ok((public_key, private_key.sign(fingerprint)))
  }
}
