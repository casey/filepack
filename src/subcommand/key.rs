use super::*;

pub(crate) fn run(options: Options) -> Result {
  let key_dir = options.key_dir()?;

  let key = KeyName::master();

  let public_key = PublicKey::load(&key_dir.join(key.public_key_filename()))?;

  {
    let private_key = PrivateKey::load(&key_dir.join(key.private_key_filename()))?;

    ensure! {
      private_key.public_key() == public_key,
      error::KeyMismatch {
        key,
      },
    }
  }

  println!("{public_key}");

  Ok(())
}
