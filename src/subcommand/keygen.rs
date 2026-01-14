use super::*;

pub(crate) fn run(option: Options) -> Result {
  let key_dir = option.key_dir()?;

  #[cfg(unix)]
  let key_dir_exists = filesystem::exists(&key_dir)?;

  filesystem::create_dir_all(&key_dir)?;

  #[cfg(unix)]
  if key_dir_exists {
    let mode = filesystem::mode(&key_dir)?;

    ensure! {
      mode.trailing_zeros() >= 6,
      error::KeyDirPermissions { path: &key_dir, mode },
    }
  } else {
    filesystem::set_mode(&key_dir, 0o700)?;
  }

  let private_path = key_dir.join(MASTER_PRIVATE_KEY);

  ensure! {
    !filesystem::exists(&private_path)?,
    error::PrivateKeyAlreadyExists { path: private_path },
  }

  let public_path = key_dir.join(MASTER_PUBLIC_KEY);

  ensure! {
    !filesystem::exists(&public_path)?,
    error::PublicKeyAlreadyExists { path: public_path },
  }

  let private_key = PrivateKey::generate();

  filesystem::write(&private_path, format!("{}\n", private_key.display_secret()))?;

  #[cfg(unix)]
  filesystem::set_mode(&private_path, 0o600)?;

  let public_key = private_key.public_key();

  filesystem::write(&public_path, format!("{public_key}\n"))?;

  Ok(())
}
