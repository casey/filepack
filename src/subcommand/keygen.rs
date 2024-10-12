use super::*;

pub(crate) fn run(option: Options) -> Result {
  let key_dir = option.key_dir()?;

  filesystem::create_dir_all(&key_dir)?;

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

  let public_key = private_key.public_key();

  filesystem::write(&public_path, format!("{public_key}\n"))?;

  Ok(())
}
