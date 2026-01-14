use super::*;

#[derive(Parser)]
pub(crate) struct Key {
  #[arg(default_value = DEFAULT_KEY, help = "Print public key <KEY>", long)]
  key: KeyName,
}

impl Key {
  pub(crate) fn run(self, options: Options) -> Result {
    let key_dir = options.key_dir()?;

    let public_key = PublicKey::load(&key_dir.join(self.key.public_key_filename()))?;

    {
      let private_key = PrivateKey::load(&key_dir.join(self.key.private_key_filename()))?;

      ensure! {
        private_key.public_key() == public_key,
        error::KeyMismatch {
          key: self.key,
        },
      }
    }

    println!("{public_key}");

    Ok(())
  }
}
