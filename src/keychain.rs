use super::*;

pub(crate) struct Keychain {
  pub(crate) keys: BTreeMap<KeyName, PublicKey>,
  pub(crate) path: Utf8PathBuf,
}

impl Keychain {
  pub(crate) fn generate_key(&mut self, name: &KeyName) -> Result {
    if !filesystem::exists(&self.path)? {
      filesystem::create_dir_all_with_mode(&self.path, 0o700)?;
    }

    ensure! {
      !self.keys.contains_key(name),
      error::PublicKeyAlreadyExists { path: self.path.join(name.public_key_filename()) },
    }

    let private_path = self.path.join(name.private_key_filename());

    ensure! {
      !filesystem::exists(&private_path)?,
      error::PrivateKeyAlreadyExists { path: private_path },
    }

    let private_key = PrivateKey::generate();

    filesystem::write_with_mode(
      &private_path,
      format!("{}\n", private_key.display_secret()),
      0o600,
    )?;

    let public_key = private_key.public_key();

    let public_path = self.path.join(name.public_key_filename());

    filesystem::write(&public_path, format!("{public_key}\n"))?;

    self.keys.insert(name.clone(), public_key);

    Ok(())
  }

  pub(crate) fn load(options: &Options) -> Result<Self> {
    let path = options.data_dir()?.join("keychain");

    if !filesystem::exists(&path)? {
      return Ok(Self {
        keys: default(),
        path,
      });
    }

    let mode = filesystem::mode(&path)?;

    ensure! {
      mode.is_secure(),
      error::KeychainPermissions { mode, path },
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
        return Err(error::KeychainUnexpectedDirectory { path }.build());
      }

      let Some(extension) = path.extension() else {
        return Err(error::KeychainUnexpectedFile { path }.build());
      };

      let key_type = extension
        .parse::<KeyType>()
        .ok()
        .context(error::KeychainUnexpectedFile { path })?;

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

    Ok(Self { keys, path })
  }

  pub(crate) fn identifier_public_key<'a>(
    &'a self,
    identifier: &'a KeyIdentifier,
  ) -> Result<&'a PublicKey> {
    match identifier {
      KeyIdentifier::Literal(key) => Ok(key),
      KeyIdentifier::Name(name) => self.public_key(name),
    }
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
