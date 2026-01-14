use super::*;

pub(crate) fn run(option: Options) -> Result {
  let dir = option.key_dir()?;

  let exists = filesystem::exists(&dir)?;

  filesystem::create_dir_all(&dir)?;

  if exists {
    let mode = filesystem::mode(&dir)?;

    ensure! {
      mode.is_secure(),
      error::KeyDirPermissions { path: &dir, mode },
    }
  } else {
    filesystem::set_mode(&dir, 0o700)?;
  }

  let private_path = dir.join(MASTER_PRIVATE_KEY);

  ensure! {
    !filesystem::exists(&private_path)?,
    error::PrivateKeyAlreadyExists { path: private_path },
  }

  let public_path = dir.join(MASTER_PUBLIC_KEY);

  ensure! {
    !filesystem::exists(&public_path)?,
    error::PublicKeyAlreadyExists { path: public_path },
  }

  let private_key = PrivateKey::generate();

  filesystem::write(&private_path, format!("{}\n", private_key.display_secret()))?;

  filesystem::set_mode(&private_path, 0o600)?;

  let public_key = private_key.public_key();

  filesystem::write(&public_path, format!("{public_key}\n"))?;

  Ok(())
}
