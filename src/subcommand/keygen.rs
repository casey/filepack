use super::*;

pub(crate) fn run(option: Options) -> Result {
  let key_dir = option.key_dir()?;

  fs::create_dir_all(&key_dir).context(error::Io { path: &key_dir })?;

  let private_path = key_dir.join(MASTER_PRIVATE_KEY);

  ensure! {
    !private_path.try_exists().context(error::Io { path: &private_path})?,
    error::PrivateKeyAlreadyExists { path: private_path },
  }

  let public_path = key_dir.join(MASTER_PUBLIC_KEY);

  ensure! {
    !public_path.try_exists().context(error::Io { path: &public_path})?,
    error::PublicKeyAlreadyExists { path: public_path },
  }

  let private_key = PrivateKey::generate();

  fs::write(&private_path, format!("{}\n", private_key.display_secret()))
    .context(error::Io { path: private_path })?;

  let public_key = private_key.public_key();

  fs::write(&public_path, format!("{public_key}\n")).context(error::Io { path: public_path })?;

  Ok(())
}
