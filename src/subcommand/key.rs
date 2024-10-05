use super::*;

pub(crate) fn run(options: Options) -> Result {
  let keys = options.keydir()?;

  let public_key = PublicKey::load(&keys.join(MASTER_PUBLIC_KEY))?;

  let private_key = PrivateKey::load(&keys.join(MASTER_PRIVATE_KEY))?;

  ensure! {
    private_key.public_key() == public_key,
    error::KeyMismatch {
      public_key: MASTER_PUBLIC_KEY,
      private_key: MASTER_PRIVATE_KEY,
    },
  }

  println!("{public_key}");

  Ok(())
}
